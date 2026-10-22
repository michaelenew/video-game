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
use crate::aim::{self, Contact, Path, Scene, Targets};
use crate::arena::{self, Arena, ArenaId, Mark, Solid, Terrain};
use crate::bolt::{self, Flight, MAX_BOLTS};
use crate::bulwark;
use crate::camera;
pub use crate::class::Shield;
use crate::class::{self, Class, Force, Form, Ghost, Mechanic, Met};
use crate::critter::{self, Critters};
use crate::debris::{self, MAX_DEBRIS, Shrapnel};
use crate::dual;
use crate::effects::{Effect, EffectKind, GRASP_ARMS, LOTUS_BLADES, MAX_EFFECTS, quarry_victim};
use crate::fixed::Fx;
use crate::gust::{self, Gale, MAX_GUSTS};
use crate::hazard::{self, Floor};
use crate::input::{Destination, Input, Travel};
use crate::lore::Lore;
use crate::math::V3;
use crate::monster::{
    self, Doing, Herd, MAX_MONSTERS, Monster, Quarry, Senses, mount_of, mount_part, mount_slot,
};
use crate::moves;
use crate::objective;
use crate::pack::{self, Pack};
use crate::region::{Body, Grid, RegionId};
use crate::shadow;
use crate::species::SpeciesId;
use crate::stones::{self, Field};
use crate::tuning as t;

pub const MAX_PLAYERS: usize = 4;

// A fighter's index travels as a byte in a hundred places -- who threw this,
// who is held by whom -- beside sentinels that are not anybody: the creature
// ([`QUARRY`]), nailed, nobody. Every seat has to stay clear of them.
const _: () = assert!(
    MAX_PLAYERS < QUARRY as usize,
    "a seat's byte would be the creature's"
);
pub type PlayerId = usize;

/// **A set of fighters**: one bit per seat, as wide as the seats need. What
/// every "already struck by", "already hit" and "seen by" mask is, so none
/// can be a `u8` that the ninth seat silently shares with the first.
pub type Fighters = crate::bits::Bits<{ crate::bits::words_for(MAX_PLAYERS) }>;

const GROUND_Y: Fx = Fx::ZERO;

/// Move slots are positional and mean the same thing on every class, which is
/// what lets one control scheme drive six kits. See `controls.md`.
/// `held_by` when nobody is holding you.
pub const NOBODY: u8 = u8::MAX;
/// `Player::held_by` for a body the Blood mage's Nail has pinned in the air:
/// held by nobody who drags it anywhere, and -- unlike a creature's spikes,
/// which hold a fighter where they stand -- held **off the floor**, gravity
/// and all. See `moves::blood::NAIL`.
pub const NAILED: u8 = u8::MAX - 1;

/// `Player::blinked` when she did not blink this frame. Past every effect
/// slot there can be: a slot is sixteen bits since the pool is sized by the
/// seats.
pub const NO_POOL: u16 = u16::MAX;
/// `Player::vessel` when the Blood mage has no charge stored.
pub const NO_VESSEL: u8 = u8::MAX;
/// `Player::charging_stone` when the Elementalist is holding no stone.
pub const NO_STONE: u8 = u8::MAX;

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

/// A jump attack's window and its trip off the ground, for a class whose
/// mechanic does not already keep them. See [`Player::rise`].
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Rise {
    /// Frames left in which a click is a takeoff rather than the move it would
    /// otherwise be. Held open while the jump button is down on the floor.
    pub window: u8,
    /// Frames since the feet left the floor. A click in the first few is still
    /// the takeoff from the floor -- `tuning::floor_grace`, the Champion's.
    pub lifted: u8,
    /// This trip off the ground has had its takeoff.
    pub used: bool,
    /// The height of what her feet were last on -- the floor, or the top of
    /// a stone. A takeoff clicked a frame or two into the jump is put back
    /// here (`back_to_the_floor`); asking the arena for its floor instead put
    /// somebody jumping off a stone back on the ground under it.
    pub floor: Fx,
    /// The stone her feet were last on, as `aim::Contact::Stone` counts them,
    /// plus one; nought for the floor. The earth jump asks it when her feet
    /// have left the stone's edge during the jump's wind-up (2026-10-18, from
    /// play: half off a stone and walking, she raised a new stone in the air
    /// instead of pushing off the one she stood on).
    pub stone: u8,
}

/// **A wall, as a fighter met it in the air**: a point on its face -- where
/// the body's middle was, held a body's radius off it -- and which way it
/// faces, level. Enough to push away from it and to tell whether the next
/// wall is the same one (`Wall::same_as`).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Wall {
    pub at: V3,
    /// Out of the wall, a unit vector; zero for no wall.
    pub normal: V3,
}

impl Wall {
    pub const NONE: Wall = Wall {
        at: V3::ZERO,
        normal: V3::ZERO,
    };

    pub fn is_some(self) -> bool {
        self.normal != V3::ZERO
    }

    /// Is `other` this wall: facing the same way, within
    /// `tuning::wall_same_turn`, and its face no further than
    /// `tuning::wall_same_gap` from this one's? Measured across the face, so
    /// anywhere along one long wall is the same wall.
    pub fn same_as(self, other: Wall) -> bool {
        if !self.is_some() || !other.is_some() {
            return false;
        }
        let facing = self.normal.dot(other.normal);
        let across = other.at.sub(self.at).dot(self.normal).abs();
        facing.raw() >= t::wall_same_turn().raw() && across.raw() <= t::wall_same_gap().raw()
    }
}

/// **Which way the stick points at a wall** (2026-10-16): the stick as it
/// would walk you on the floor, by the camera, against the way the wall
/// faces. See [`lean_against`].
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Lean {
    /// Nothing pressed, or a stick that cancels itself out.
    Still,
    /// Within `tuning::wall_cone` of straight into the wall.
    Into,
    /// Within it of straight out of the wall, along its face's normal.
    Away,
    /// Anything between: along the wall, this way (a level unit vector on
    /// its face).
    Along(V3),
}

/// **Where the stick points, against a wall facing `normal`** (level, unit,
/// out of the wall): into it, away from it or along it, with
/// `tuning::wall_cone` either side of straight in and straight out. The one
/// reading of the stick against a wall, for the wall slide and the ledge.
pub fn lean_against(input: Input, normal: V3) -> Lean {
    let (ax, az) = input.move_axis();
    if ax == 0 && az == 0 || normal == V3::ZERO {
        return Lean::Still;
    }
    let dir = move_dir(input.aim_turns(), ax, az);
    let out = dir.x.mul(normal.x).add(dir.z.mul(normal.z));
    let cone = t::wall_cone();
    if out.raw() >= cone.raw() {
        Lean::Away
    } else if out.neg().raw() >= cone.raw() {
        Lean::Into
    } else {
        // Along the face: the stick less its part out of it, which is never
        // zero here -- a stick inside neither cone is not along the normal.
        let along = V3::new(
            dir.x.sub(normal.x.mul(out)),
            Fx::ZERO,
            dir.z.sub(normal.z.mul(out)),
        );
        Lean::Along(along.normalized())
    }
}

/// **The ledge a fighter is holding** (2026-10-16): which edge, and how far
/// into holding it. [`Ledge::NONE`] when it holds none. See [`hang_on`].
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Ledge {
    /// The edge held: where the hands are, and which way its face looks.
    pub edge: crate::arena::Edge,
    /// Where the body was when it caught the edge: the catch settles it from
    /// here into the hang.
    pub from: V3,
    pub grip: Grip,
    /// Frames into the grip.
    pub clock: u8,
    /// Frames since the catch, whatever the grip.
    pub since: u8,
    /// Set once the stick has been anywhere but into the wall since the
    /// catch. **Into the wall is both "catch it" and "climb it"**, so a stick
    /// held into the wall through the catch climbs only once
    /// `tuning::ledge_hold_on` has run -- time to decide to hang -- while
    /// one let go and pressed in again climbs at once.
    pub fresh: bool,
}

/// How a fighter is holding a ledge.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Grip {
    /// Settling into the hang, from wherever the catch found the body.
    Catch,
    /// Hanging: still, or hand over hand along the edge.
    Hang,
    /// Climbing up onto the top.
    Climb,
}

impl Ledge {
    pub const NONE: Ledge = Ledge {
        edge: crate::arena::Edge {
            at: V3::ZERO,
            normal: V3::ZERO,
        },
        from: V3::ZERO,
        grip: Grip::Hang,
        clock: 0,
        since: 0,
        fresh: false,
    };

    pub fn is_some(self) -> bool {
        self.edge.normal != V3::ZERO
    }
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
    /// Frames left of a Grasp haul, and where it is pulling her.
    ///
    /// **The Blood mage's movement, behind `tuning::grasp_hauls`.** A Grasp
    /// whose four arms converge on a wall, a structure or the creature rather
    /// than on a person has nothing to drag, so it drags *her* -- a grappling
    /// hook made out of an ability she already owns. Driven rather than
    /// thrown: gravity and the arena are off for the duration, the way they are
    /// for the Reaver's dash, so the line she travels is the line the arms
    /// went out along.
    ///
    /// Zero for everybody else, always, and zero for her whenever the flag is
    /// off.
    pub haul: u16,
    pub haul_to: V3,
    /// How fast that haul reels, in metres a second.
    ///
    /// Per haul rather than one knob, because the two things that arm one want
    /// opposite guarantees. The Grasp wants a constant **speed**: you are being
    /// reeled in, and how long it takes is however far away the wall was. The
    /// blink wants a constant **time**: it is a fixed distance crossed in a
    /// fixed handful of frames, and a blink whose duration depended on how much
    /// of its range the geometry allowed would be a different move at every
    /// wall. One field answers both -- see `begin_blink` and
    /// `World::haul_to_the_grasp`.
    pub haul_speed: Fx,
    /// How many aerials have hung this airtime. Reset on landing.
    ///
    /// **The air gives you less each time you ask.** A hang is per-move and
    /// costs nothing but the move, so a class with two interchangeable pokes
    /// can alternate them and never come down -- the repeat lockout stops one
    /// move being mashed and has nothing to say about two. The Dual mage's
    /// autos are exactly that pair, and hers hang longer than anybody's on
    /// purpose, so the falloff is what keeps the floatiest poke in the game
    /// from being a hover. See `tuning::air_stall_falloff`.
    pub air_stalls: u8,
    /// How many Fire carpets have hung the Elementalist this airtime. Reset
    /// with `air_stalls`.
    ///
    /// **Her carpet hangs her long**, so the Updraft into it comes up through
    /// it, and the shared falloff alone let a jump of carpets float her most
    /// of a second. Each carpet after the first is worth
    /// `tuning::carpet_hang_falloff` of the one before, counted on its own,
    /// so another aerial first does not take the long hang off the first
    /// carpet (2026-10-21, from play).
    pub carpets: u8,
    /// An airdodge has been spent this airtime. Reset on landing.
    ///
    /// Without it, airdodging repeatedly is free flight: each one is a fresh
    /// burst of horizontal speed with invulnerability attached.
    pub air_dodged: bool,
    /// The last wall this fighter's body was against in the air.
    ///
    /// Frames since are [`Player::wall_since`]; the slide and the wall jump
    /// read both. See [`cling_to_the_wall`].
    pub wall: Wall,
    /// Frames since the resolve last held this fighter off a wall in the air:
    /// zero while against one, `u8::MAX` on the floor or never.
    pub wall_since: u8,
    /// The wall the last wall jump left, this airtime, and where the body was
    /// when it jumped; or [`Wall::NONE`].
    ///
    /// **Off the same wall again only from lower down**
    /// (`tuning::wall_jump_drop`). Without it a wall jump, an air-steer back
    /// with an aerial's hang to buy the time, and another wall jump is a
    /// ladder. Cleared on landing.
    pub wall_jumped: Wall,
    /// How many wall jumps this airtime, for `tuning::wall_jump_falloff`.
    /// Reset on landing.
    pub wall_jumps: u8,
    /// **A press of jump kept for a wall** (`tuning::wall_jump_buffer`):
    /// frames it has left, or zero. Set by a press in the air that did
    /// nothing else, spent by the wall jump it becomes.
    pub wall_press: u8,
    /// **On a wall slide**, as the stick last left it (`keep_sliding`):
    /// started by touching a wall while falling with the stick anywhere but
    /// away from it, ended by pushing away from it.
    pub sliding: bool,
    /// Frames before the slide may start or end again
    /// (`tuning::wall_slide_flip`).
    pub slide_flip: u8,
    /// The ledge this fighter is holding, or [`Ledge::NONE`]. See [`hang_on`].
    pub ledge: Ledge,
    /// Frames before a ledge can be caught again (`tuning::ledge_regrab`).
    pub ledge_regrab: u8,
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
    /// How many marks the Reaver's shadow has put on this fighter.
    ///
    /// **On the victim, not on her**, because the victim is who has to read
    /// it: pips over a body are the counterplay made visible, and a fighter
    /// watching his own marks climb knows what is coming and where from. Every
    /// hit the shadow lands from the field adds one, up to `tuning::mark_cap`;
    /// the first swing of hers to connect inside the window a dash to the
    /// shadow opens spends them all. See `docs/design/shadow-reaver-v2.md`.
    ///
    /// Zero for everybody nobody is marking, which is everybody in a match
    /// without a Reaver in it.
    pub marks: u8,
    /// Frames until the next mark fades. Reset by every new mark, so a target
    /// who gets away from the shadow cleans himself one mark at a time.
    pub mark_clock: u16,
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
    /// move she is in the middle of -- the bar of the force that move is made
    /// of. [`Fx::ONE`] for everybody else, and meaningless while she is not
    /// throwing anything.
    ///
    /// **A cast is worth where you were standing when you pressed the button**,
    /// not where the cast's own push has since taken you. The two are different
    /// numbers because throwing anything at all goads a bar, on the press,
    /// and a finisher goads it a long way -- so a Judgement read live would be
    /// worth its own push, and the one place in the kit that is supposed to be
    /// embarrassing from empty would be the least embarrassing thing there.
    /// It would also mean the bars on the HUD never matched what the player got.
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
    pub blinked: u16,
    /// Frames of **bleed** left, from a Blood mage's Haemorrhage. Every
    /// `tuning::bleed_tick` frames it takes `tuning::bleed_damage` and spills
    /// that under you wherever you are standing, so a bleeding fighter walks
    /// a trail of her pools behind them -- which is the point: a spike on any
    /// of them chains along the trail. See `World::step_bleeds`.
    pub bleeding: u16,
    /// Whose bleed it is, so the pools are hers. `u8::MAX` for nobody.
    pub bled_by: u8,
    /// This fighter's haul ended this frame: the Grasp's arms have brought
    /// them to their captor's feet. `drag_the_held` has only the bodies and
    /// cannot see the floor, so it leaves the fact here for `advance` to
    /// spend on the same frame -- the Blood mage drinks the pool the victim
    /// has just been hauled onto. In the snapshot for the reason every edge
    /// is: a frame re-simulated must drink the same pool.
    pub hauled_in: bool,
    /// Downward speed banked by a spike, cashed in when this fighter lands.
    ///
    /// The Champion's aerial hammer drives an airborne target into the ground,
    /// and the ground is meant to be the larger half of that: the hit does its
    /// damage, and then the landing does more. Carried as the speed rather
    /// than as a flag so the cost scales with how far they had to fall --
    /// bounded above by terminal velocity, which is what makes the worst case
    /// knowable.
    pub slam: Fx,
    /// The fastest the Bulwark has fallen during Slam's wind-up, in metres per
    /// second. What the kit's fall-speed scaling reads: a slam out of a leap is
    /// worth more than one thrown standing. Zero outside a Slam. See
    /// `crate::bulwark::track_fall`.
    pub slam_fall: Fx,
    /// Upward speed a grab banked, spent when the hold lets go.
    ///
    /// A held fighter has their velocity zeroed every frame by
    /// `drag_the_held`, so a launch handed to them on contact would be gone
    /// before it moved them. It waits instead: held, you go where your captor
    /// goes; let go, and you are thrown. See `docs/design/stun.md`.
    pub thrown: Fx,
    /// Blows from a fighter taken without getting a turn back: the length of
    /// the string this fighter is in. Each one keeps less of the hitstun
    /// swell (`tuning::combo_decay`), and it is cleared the moment they are
    /// free to act again.
    pub taken: u8,

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
    /// Clicks that must come up before they count again: one whose charge
    /// released itself at its cap with the button still down (2026-10-21,
    /// from play). Without it, holding a full Strike threw it, and then threw
    /// a second pillar's whole wind-up the moment the first one's recovery
    /// ended. Input bits; cleared as each button comes up. The
    /// Elementalist's only, so far: see `step_channel`.
    pub spent_clicks: u16,
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
    /// Frames left of the impact freeze: this body holds exactly still,
    /// action and clocks and position, until it runs out.
    ///
    /// Set on both bodies when a blow connects -- see [`Move::hitstop`] -- so
    /// the pair of them stop together and start together, and no frame
    /// advantage moves. The knockback is already in `vel` when it starts and
    /// is spent after, which is what makes a freeze followed by a slide read
    /// as a single impact rather than as two events. In the snapshot for the
    /// same reason every clock is: a rollback that forgot it would replay the
    /// hit without the stop.
    ///
    /// The renderer reads it too, for the one thing the simulation cannot do
    /// about weight: shake the view. See `view::camera`.
    ///
    /// [`Move::hitstop`]: crate::moves::Move::hitstop
    pub frozen: u8,
    /// Buttons pressed at any point during the freeze, delivered on the first
    /// frame after it. A freeze is a pause in the fight, not a pause in the
    /// player, and a click that started and finished inside one would
    /// otherwise have been thrown away.
    pub frozen_bits: u16,
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

    /// How high this fighter's feet are off the floor under them **as the
    /// camera may know it**: nought on anything, the height in the air, and
    /// rate-limited on the way up (`camera::clear_step`) so that a ledge
    /// stepped off eases the eye out rather than throwing it. What the
    /// camera's bubble measures the room under the eye from -- looking up off
    /// the ground, the eye is let back out of the head as far as that room
    /// allows (`camera::ball`). Stepped beside [`Player::aloft`], for its
    /// reasons.
    pub clear: Fx,

    /// Which part of the creature this fighter is standing on, or
    /// `monster::NO_MOUNT`.
    ///
    /// **While this is set, `local` is the authoritative position and `pos` is
    /// derived from it.** Unmounted it is the other way round. That one rule is
    /// the whole of why the creature can spin under a rider without sliding
    /// them off: a world position would have to be corrected for the rotation
    /// every frame, and a body-space one never has to be corrected at all.
    pub mount: monster::Mount,
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
    /// Which of the creature's clips the surface underfoot was last measured
    /// on. The pose is a lookup, so a change of clip is a cut, not a motion,
    /// and the grip test must not read it as one. See `step_rider`.
    pub ride_clip: u8,

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
    /// **How far below the shoulder a standing swing meets something short**,
    /// in metres (zero or below), from `aim::stoop`. Zero whenever only
    /// fighters are about. A swing's path carries it already; the Dual mage's
    /// wing, which lies flat at the shoulder when she stands, reads it here so
    /// that it comes down onto a gnawer as a punch would. See `aim::stoop`.
    pub stoop: Fx,
    /// Which of the Elementalist's structure slots holds the stone she raised
    /// with the mechanic key still down, or [`NO_STONE`].
    ///
    /// Set on the press. While the key stays down it is an ordinary raise
    /// until the stone's churn is about to end; on that frame the hold takes
    /// over (`Action::Channel` on Fissure's slot) and the stone is kept
    /// churning until the key comes up, when the crack races from it. Cleared
    /// the frame the crack comes out, the frame the key is released early, and
    /// the frame anything else takes the action -- a hit, or a different move.
    /// See [`hold_the_churn`] and `moves::Charge::Crack`.
    pub charging_stone: u8,
    /// Were `F` and `R` down last frame? The press edges for the two keys,
    /// in the snapshot for the reason `mechanic_held` is: rollback re-runs
    /// these frames, and an edge remembered outside it disappears the first
    /// time one is replayed.
    pub f_held: bool,
    pub r_held: bool,
    pub side_b_held: bool,
    /// Was the Elementalist's earth click down last frame? Raise is an
    /// instant on the press, as it was on `E`, so it needs an edge of its own:
    /// level-triggered, a held click would spend all three stones on one spot
    /// in three frames -- the bug the mechanic key's own edge was made for.
    pub earth_held: bool,
    /// **The jump attack's window and its trip**, for a class that is not the
    /// Champion -- today the Elementalist, whose three clicks with space are
    /// three takeoffs. The same three numbers the Champion keeps inside its
    /// mechanic (`Mechanic::Forms { takeoff, lifted, rise_used }`): frames left
    /// in which a click leaves the floor, frames since the feet left it, and
    /// whether this trip's takeoff is spent. See [`rise_window`].
    pub rise: Rise,
    /// Where the Elementalist's Air ball is being raised, while she holds it.
    ///
    /// Set on the press, from the crosshair, and read by the world on the
    /// frame the ball appears and by the release's aim, which sends the ball
    /// from here toward where the crosshair is then. A position, so it is
    /// moved by `World::shift` with everything else a fighter keeps.
    pub ball_at: V3,
    /// **The Blood mage's charge**: the tier the move she has out was let go
    /// at -- one to three -- or nought for anything untiered. See
    /// `moves::Charge::Tier`.
    pub tier: u8,
    /// **Her vessel**: a charge stored by a jump or a dodge out of its hold,
    /// to be let go later with the same click -- the move, or [`NO_VESSEL`] --
    /// and the tier it had reached. It bleeds her while it holds anything.
    pub vessel: u8,
    pub vessel_tier: u8,
    /// Has the click that stored it come up since? A stored charge lets go
    /// on a fresh press, never on the hold that charged it.
    pub vessel_ready: bool,
    /// What the cast she has out cost her, owed to the floor: it lands as a
    /// stain where the cast does, on its first active frame. Nought when
    /// nothing is owed.
    pub paid: u16,
    /// Frames before the Blood mage's grey starts to fade: set to
    /// `tuning::grey_grace` every time grey opens, counted down after.
    pub grey_rest: u8,
    /// What a finisher of hers is spending: all the grey she
    /// had when it began, held from the frame the hold ran out to the frame
    /// it lands. Nought otherwise.
    pub fuel: u16,
    /// **What a finisher owes her** (2026-10-14): the grey it burned when it
    /// began, paid back into red as the finisher lands
    /// (`tuning::finisher_repays`, by way of `Player::heal`). Nought once it
    /// is repaid; replaced by the next finisher's.
    pub owed: u16,
    /// **Standing on her own ground** (2026-10-18): the Blood mage's feet on
    /// the floor -- or sunk in it, swimming -- inside a patch of her ground,
    /// a stain of hers or a heart's reach. Worked out once a frame, before
    /// anybody moves (`World::mark_the_blood`), because the input, the
    /// thralls and the renderer all ask it. False for everybody else.
    pub on_blood: bool,
    /// **What she has dealt toward a heart** (2026-10-18): every point of
    /// damage she lands -- a blow, an eruption, a bite, a bleed -- up to
    /// `tuning::heart_meter_full`. Full, `E` plants a heart, and planting
    /// empties it. Nought for everybody else.
    pub heart_meter: u16,
    /// **Swimming in her ground** (2026-10-18): [`SWIM_UNDER`] while sunk
    /// into it with `shift` held, fast and short; [`SWIM_SPENT`] once she
    /// has come up with `shift` still down, until it comes up -- a held
    /// shift is not a fresh dodge. Only while she is free and on her ground;
    /// a click, a jump, a hit, letting go or leaving it brings her up. See
    /// `docs/design/exploration/0021_blood_mage_territory.md` §5.
    pub swim: u8,
    /// Which blow of a string of Sweeps this is: nought for the first, one for
    /// the second (back the other way), two for the third (heavier).
    /// [`STRINGING`] is set from the press that strings it to the move it
    /// starts.
    pub string: u8,
    /// **The Dual mage's curse on this body** (2026-10-12): how many stacks
    /// and whose, packed as `dual::stacks` and `dual::cursed_by` read it, and
    /// the clock until the next stack fades. Her light purges it -- see
    /// `dual::purge_damage`. The bytes the hex was, because the snapshot is at
    /// its cap.
    pub curse: u8,
    pub curse_clock: u16,
    /// Which of her structure slots the Elementalist's dodge is breaking
    /// through this frame, or [`NO_STONE`].
    ///
    /// The dodge branch has only her body and cannot break a stone or leave
    /// ground behind, so it marks the slot here for `advance` to spend on the
    /// same frame -- the shape `blinked` has, for the same reason: a frame
    /// re-simulated must break the same stone.
    pub breaking: u8,
    /// **How far this fall goes past the free height** (bestiary P6), in
    /// metres: how high the last thing stood on was above
    /// `tuning::fall_free`, lowered by any push upward on the way down, and
    /// zero for any fall that started lower than that.
    ///
    /// One rule for the whole game, keyed to the **height fallen** rather than
    /// the landing speed -- terminal velocity arrives after four metres, so
    /// speed cannot tell a ledge from the sky. Measured from the last
    /// *footing* -- the floor, a solid, a creature's back -- because a
    /// fighter's own jump is not a fall: a hop off a platform, the Champion's
    /// leap, the Dual mage flying are all free however high they go. A push
    /// up on the way down -- a second jump, a wing beat, an Updraft, a
    /// launch -- starts the fall again from where it happened, never higher.
    /// A creature that carries a fighter up and lets go sets it where it let
    /// go ([`Player::falling_from`]).
    ///
    /// Kept as the excess over the free height rather than the height itself
    /// so that it is **zero for every fall that cannot hurt**: in the proving
    /// ground nothing stands higher than the Ridgeback's back, which is under
    /// the free height, so this never leaves zero there and the pinned hunts
    /// hash exactly as they did. See `World::land`.
    pub fall_over: Fx,
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
    pub repeat_lock: [u16; moves::LOCKED_SLOTS],
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
    pub reactivate_lock: [u16; moves::LOCKED_SLOTS],
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
            health: t::health_of(class),
            ..Player::default()
        }
    }

    /// What this fighter's bar holds when it is full. Per class -- see
    /// `tuning::health_of`.
    pub fn full_health(&self) -> i32 {
        t::health_of(self.class)
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
        // **A heart is earned** (2026-10-18): only with her meter full.
        if self.class == Class::BloodMage && kind == moves::blood::HEART {
            return self.heart_ready();
        }
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
            // Nothing on the class is gated by the bars: what they change is
            // power and what her body can do, never availability.
            Mechanic::Forms { .. } | Mechanic::Blood | Mechanic::Meter { .. } => true,
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
                // hanging or still chasing. The fire pillar: out for as long as
                // it burns -- one of hers at a time. A tornado Cataclysm tore
                // loose from it is not a pillar any more, and does not count.
                SLOT_SPECIAL => {
                    let kind = match self.class {
                        Class::Elementalist => crate::effects::EffectKind::FirePillar,
                        // The Hemoplague: one cloud of hers at a time.
                        Class::BloodMage => crate::effects::EffectKind::Plague,
                        _ => crate::effects::EffectKind::GuillotineLotus,
                    };
                    scene
                        .effects
                        .iter()
                        .flatten()
                        .any(|e| e.kind == kind && e.owner == who as u8)
                }
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
        // The Reaver's shadow, just collected by a dash, rests before anything
        // on right click sends it again (2026-10-18). See `Shadow::rest`.
        if self.class == Class::ShadowReaver
            && moves::reaver::places_the_shadow(kind)
            && shadow::resting(self)
        {
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
            self.rest_the_grey();
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
        // The Dual mage standing in her own sigil is warded: every wound is
        // cut on its way in, whoever dealt it. See `dual::through_the_ward`.
        let damage = dual::through_the_ward(self, damage);
        let dealt = damage.min(self.health).max(0);
        self.health -= dealt;
        if self.class.wounds_go_grey() {
            self.grey += dealt;
            self.rest_the_grey();
        }
        dealt
    }

    /// **Can nothing touch this fighter right now?** A dodge's opening
    /// frames, or the Dual mage's Veil (2026-10-12). The one question every
    /// hit asks before it lands -- `Action::invulnerable` is only the dodge's
    /// half of it.
    pub fn untouchable(&self) -> bool {
        self.action.invulnerable() || dual::veiled(self)
    }

    /// Grey just opened: it holds for `tuning::grey_grace` before it fades.
    fn rest_the_grey(&mut self) {
        self.grey_rest = t::grey_grace().min(u8::MAX as u16) as u8;
    }

    pub fn heal(&mut self, amount: i32) {
        // **The Blood mage is healed only out of what a finisher owes her**
        // (`Player::owed`): her finishers' hits are the one leech she has,
        // and they can give back what the finisher burned and no more.
        let amount = if self.class.wounds_go_grey() {
            let took = amount.min(self.owed as i32).max(0);
            self.owed -= took as u16;
            took
        } else {
            amount
        };
        // Never above the bar, and never into the grey: a heal from anywhere
        // but a pool cannot close a wound this class has open.
        if amount > 0 && self.health > 0 {
            self.health = (self.health + amount).min(self.full_health() - self.grey);
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

    /// **Drink a stain**: up to `amount` of grey turned back into red. What
    /// her grey could not hold is gone: the Well that kept it was cut
    /// (2026-10-19). What came back as red is given back.
    pub fn absorb(&mut self, amount: i32) -> i32 {
        self.drink(amount)
    }

    /// How much of the bar is grey, nought to one. What the scythe reads.
    pub fn grey_share(&self) -> Fx {
        let max = self.full_health().max(1);
        Fx::ratio(self.grey.clamp(0, max), max)
    }

    /// Standing on the creature.
    pub fn aboard(&self) -> bool {
        self.mount != monster::NO_MOUNT
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
    /// Has she dealt enough to plant a heart?
    pub fn heart_ready(&self) -> bool {
        self.heart_meter as i32 >= t::heart_meter_full()
    }

    /// Is she sunk in her ground, swimming?
    pub const fn swimming(&self) -> bool {
        self.swim == SWIM_UNDER
    }

    pub fn hurt_height(&self) -> Fx {
        // Sunk in her ground, she is a ripple on it: most of what is thrown
        // at a standing body goes over her.
        if self.swimming() {
            t::swim_height()
        } else if self.crouching {
            t::body_height().mul(t::crouch_height_scale())
        } else {
            t::body_height()
        }
    }

    /// **Start this fighter's fall from `height`**: what a creature that has
    /// carried somebody up calls as it lets go -- a talon drop, a fling from a
    /// back -- so the fall is measured from where they were released rather
    /// than from the last thing they stood on.
    pub fn falling_from(&mut self, height: Fx) {
        self.fall_over = height.sub(t::fall_free()).max(Fx::ZERO);
    }

    /// Is something under the feet that a fall is measured from: the floor, a
    /// solid, a creature's back?
    pub fn footed(&self) -> bool {
        self.grounded || self.aboard()
    }
}

/// **An updraft and falling water**, after gravity has had its say.
///
/// A body in the air inside an updraft's column, below its top, is carried
/// up at `tuning::vent_rise` at least, and let go at the top to drift. That
/// is a push up, and the fall rule already lowers a fall's start for one.
///
/// A body in the air in a waterfall's falling water is **pushed down** at
/// `tuning::falls_push` at least, and its held jump is spent: the curtain is
/// not a way up (`arena::waterfall`).
///
/// (Vines went on 2026-10-16: ledges catch and walls are jumped off now, and
/// a vine was a place to get stuck in.)
fn ride_the_air(p: &mut Player, ground: &Terrain) {
    if !p.grounded && crate::valley::vent_on(ground, p.pos) {
        let rise = t::vent_rise();
        if p.vel.y.raw() < rise.raw() {
            p.vel.y = rise;
        }
    }
    if !p.grounded && crate::valley::sheet_on(ground, p.pos) {
        let push = t::falls_push().neg();
        if p.vel.y.raw() > push.raw() {
            p.vel.y = push;
        }
        p.jump_hold = 0;
    }
}

/// What touching down this frame costs: `was` the body as the frame found
/// it, `now` as the step left it, before the rule has moved
/// [`Player::fall_over`]. Zero for anything but a landing past the free
/// height. The fight report reads it too, to count what falls cost.
pub fn landing_damage(was: &Player, now: &Player) -> i32 {
    landing_damage_over(was, now, Fx::ZERO)
}

/// [`landing_damage`], in a world whose lowest floor is `lowest` rather than
/// zero: in the valley, a place below the one the world is in has floors
/// under zero, and a fall onto one of them is a fall like any other
/// ([`Terrain::lowest`]).
pub fn landing_damage_over(was: &Player, now: &Player, lowest: Fx) -> i32 {
    if was.footed() || !now.footed() || was.fall_over.raw() <= lowest.raw() {
        return 0;
    }
    let past = was.fall_over.sub(now.pos.y.max(lowest));
    if past.raw() <= 0 {
        return 0;
    }
    let damage = past.mul(Fx::from_int(t::fall_per_metre())).to_int();
    if was.vel.y.neg().raw() < t::fall_soft().raw() {
        damage / 2
    } else {
        damage
    }
}

/// **The fall rule, a frame on** (bestiary P6): `was` the fighter at the top
/// of the frame, `p` at the end of it. Returns the damage a landing this frame
/// is worth, and keeps [`Player::fall_over`].
///
/// Footing sets the height a fall starts from. In the air, a push up starts it
/// again from wherever the body is, never higher. Landing pays for what is
/// past the free height and below where it landed: a fall from the crown of
/// a shell onto its rim is the drop between them, not the crown's height.
/// Halved for a landing slower than `fall_soft`, which only the Dual mage's
/// slow fall manages.
pub fn fall_rule(was: &Player, p: &mut Player) -> i32 {
    fall_rule_over(was, p, Fx::ZERO)
}

/// [`fall_rule`], over a lowest floor of `lowest` ([`landing_damage_over`]).
pub fn fall_rule_over(was: &Player, p: &mut Player, lowest: Fx) -> i32 {
    let damage = landing_damage_over(was, p, lowest);
    let over_here = p.pos.y.sub(t::fall_free()).max(lowest);
    if p.footed() {
        p.fall_over = over_here;
    } else if p.vel.y.raw() > 0 {
        p.fall_over = p.fall_over.min(over_here);
    }
    damage
}

impl Default for Player {
    fn default() -> Self {
        Player {
            pos: V3::ZERO,
            vel: V3::ZERO,
            facing: V3::new(Fx::ONE, Fx::ZERO, Fx::ZERO),
            health: t::health_of(Class::Bulwark),
            grey: 0,
            action: Action::Free,
            grounded: true,
            jump_hold: 0,
            air_stall: 0,
            haul: 0,
            haul_to: V3::ZERO,
            haul_speed: Fx::ZERO,
            air_stalls: 0,
            carpets: 0,
            air_dodged: false,
            wall: Wall::NONE,
            wall_since: u8::MAX,
            wall_jumped: Wall::NONE,
            wall_jumps: 0,
            wall_press: 0,
            sliding: false,
            slide_flip: 0,
            ledge: Ledge::NONE,
            ledge_regrab: 0,
            hit_used: false,
            repeat_lock: [0; moves::LOCKED_SLOTS],
            reactivate_lock: [0; moves::LOCKED_SLOTS],
            class: Class::Bulwark,
            mechanic: Mechanic::Shield(Shield::Held { weight: Fx::ZERO }),
            rounds_won: 0,
            crouching: false,
            slowed: 0,
            slow_mul: Fx::ONE,
            hasted: 0,
            marks: 0,
            mark_clock: 0,
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
            hauled_in: false,
            slam: Fx::ZERO,
            slam_fall: Fx::ZERO,
            thrown: Fx::ZERO,
            taken: 0,
            stride: 0,
            air_frames: 0,
            since_landed: 0,
            parried: 0,
            spent_clicks: 0,
            crouched_for: 0,
            stun_total: 0,
            frozen: 0,
            frozen_bits: 0,
            aloft: Fx::ZERO,
            clear: Fx::ZERO,
            mount: monster::NO_MOUNT,
            local: V3::ZERO,
            carry_yaw: Fx::ZERO,
            grip_vel: V3::ZERO,
            grip_settle: 0,
            ride_clip: u8::MAX,
            beam_reach: Fx::ZERO,
            aim_path: Path::default(),
            stoop: Fx::ZERO,
            channelled: Fx::ZERO,
            charging_stone: NO_STONE,
            f_held: false,
            r_held: false,
            side_b_held: false,
            earth_held: false,
            rise: Rise::default(),
            ball_at: V3::ZERO,
            tier: 0,
            vessel: NO_VESSEL,
            vessel_tier: 0,
            vessel_ready: false,
            paid: 0,
            grey_rest: 0,
            fuel: 0,
            owed: 0,
            on_blood: false,
            heart_meter: 0,
            swim: SWIM_NONE,
            string: 0,
            curse: 0,
            curse_clock: 0,
            breaking: NO_STONE,
            fall_over: Fx::ZERO,
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
    /// **The Blood mage's ground** (2026-10-18): the patches of floor her
    /// blood has claimed. See [`crate::ground`].
    pub ground: crate::ground::Ground,
    /// Which of `ground`'s patches a heart holds (2026-10-19), as the last
    /// frame's `ground::step` found them: drawn whole, and never made way
    /// with when the table is full. See [`crate::ground::Held`].
    pub ground_held: crate::ground::Held,
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
    /// The quarry, in a hunt: up to [`MAX_MONSTERS`] creatures, each of any
    /// species. All `None` is a versus match.
    ///
    /// Two slots because the Pair are two full monsters, and every other
    /// creature is one. The second slot costs its bytes in every fight,
    /// Ridgeback included; an enum of fight shapes would be smaller and is the
    /// right thing once packs of critters want the same room (see
    /// `docs/design/bestiary.md` P3), but for two an array is honest and cheap.
    /// Everything that touches a creature walks the slots in order, so a hunt
    /// with one creature in slot zero is exactly the one-creature game.
    pub monsters: Herd,
    /// Where the fight is: one byte, and the rest is a table
    /// ([`arena::ArenaId::get`]). In the snapshot because which arena is
    /// loaded is gameplay -- every wall, floor and spawn reads it -- and a
    /// change of arena has to land on the same frame for both peers and roll
    /// back with everything else. See `docs/design/arenas.md`.
    pub arena: ArenaId,
    /// **Small bodies** -- a pack's critters, up to [`critter::MAX_CRITTERS`]
    /// of them, and whose they are. Every slot empty outside a pack fight. See
    /// `crate::critter` and `docs/design/critters.md`.
    pub critters: Critters,
    /// The brain that drives them, when there is a pack: one per fight. A
    /// creature that is only a pack (the Gnawers) has this and no monster; a
    /// creature that owns one (the Broodmother) has both.
    pub pack: Option<Pack>,
    /// **The hunt's own room**: floor hazards, the noise ring, defended
    /// things, and whatever state the fight's species keeps -- one region
    /// sized to the largest fight and laid out by whichever species this one
    /// is. See [`crate::lore`]. Blank, and not hashed, in a fight that uses
    /// none of it.
    pub lore: Lore,
    /// **A jump course's runs**, one per fighter: the furthest gate reached,
    /// the falls, the clock (`crate::course`). All zero, and not hashed, in an
    /// arena that is not a course.
    pub course: [crate::course::Run; MAX_PLAYERS],
    /// **Paused**, against a person: P on either machine, on the wire
    /// ([`Destination::Pause`]), so both stop on the same frame -- what P does
    /// for two people at one keyboard. While it is set a frame changes nothing
    /// but the frame count, which the rollback session owns. Training pauses
    /// by not ticking at all, and never sets it.
    pub paused: bool,
    /// **How many seats are played**: two, or one with the second fighter out
    /// of any hunt. The harness has always taken absent hunters out, because
    /// a fighter nobody drives is a body the creature walks over to stand on;
    /// the game does the same for its dummy. Kept here so that a fresh fight
    /// built *inside* a tick -- a trip on the wire, a restart -- seats itself
    /// ([`World::seated`]), and a replay of the inputs alone rebuilds the same
    /// fight (`crate::replay`).
    ///
    /// **Not hashed.** It is read only as a fight is built, and what it
    /// changes -- a fighter's health -- is. Against a person both seats are
    /// played and it is two on both machines.
    pub seats: u8,
    /// **The valley's journey** (`crate::valley`): whether this world is the
    /// valley, which creatures have been beaten on the way, which seam you
    /// came in by, each fighter's last cairn. All zero, and not hashed,
    /// outside it.
    pub valley: crate::valley::Journey,
    /// **The seats whose players have left the match** this frame: the
    /// rollback session's word for a dropped connection, on the wire
    /// ([`Destination::Left`]), read afresh every frame. A seat that has left
    /// is parked out of the world as an empty one is, and is not counted as
    /// anybody still in the fight -- so the one who stayed in a duel is not
    /// handed a round every few seconds by a body that is not there.
    ///
    /// It used to be nothing: a dropped player's fighter stood where they
    /// left it, a target for everybody, turned to face one way because their
    /// look on the wire had become zero.
    pub away: Fighters,
    /// **Who has pressed "Start match"** ([`Destination::Ready`]): a seat's
    /// flag flips with each press, and the frame every seat in the fight has
    /// it -- two of them at least -- the world starts the match itself
    /// ([`World::match_start`]), so every machine starts it on the same frame.
    /// A seat that leaves loses its flag. Empty in every fresh fight. Hashed
    /// only while somebody is ready, so every pinned checksum stays pinned.
    pub ready: Fighters,
    /// **The crowd field** (`crate::crowd`, exploration 0018): what the
    /// world knows of the regions round the fighters' own, fed once an
    /// epoch and read slowly. Which regions it follows is decided here, in
    /// the step, from where the fighters stand -- so every peer follows the
    /// same ones on the same frame. Hashed only when it has heard
    /// something: a fight that has not hashes as it did before there was a
    /// field.
    pub crowd: crate::crowd::Field,
    /// Eggs and trophies on the floor of this place (`crate::keep`).
    pub loose: crate::keep::Loose,
    /// What each seat picked up this frame, or was given (`crate::keep`):
    /// how a game hears of a trophy for its bag. Emptied every frame.
    pub picked: [Option<crate::keep::Item>; MAX_PLAYERS],
    /// **King of the Hill** (`crate::versus`): the capture point, both
    /// sides' clocks, and who is waiting to stand back up. Off, and not
    /// hashed, on every arena that has no hill.
    pub hill: crate::versus::Koth,
}

impl World {
    pub fn new() -> World {
        World::with_classes([Class::Bulwark; 2])
    }

    /// **Every seat's class, from as many as were named**: a fight names one
    /// class per fighter it seats -- two for a duel or a pair of hunters, up
    /// to [`MAX_PLAYERS`] -- and the seats past those are filled with the
    /// first class named, for the bodies that stand parked out of the world
    /// ([`World::seated`]). `N` is checked at compile time.
    pub fn seats_of<const N: usize>(classes: [Class; N]) -> [Class; MAX_PLAYERS] {
        const { assert!(N >= 1 && N <= MAX_PLAYERS, "one to four fighters") };
        let mut all = [classes[0]; MAX_PLAYERS];
        all[..N].copy_from_slice(&classes);
        all
    }

    /// Start a match with a chosen class per fighter: as many fighters as
    /// classes named, one to [`MAX_PLAYERS`].
    pub fn with_classes<const N: usize>(classes: [Class; N]) -> World {
        World::with_all(World::seats_of(classes)).seated(N as u8)
    }

    /// Every seat filled, from a class per seat.
    fn with_all(classes: [Class; MAX_PLAYERS]) -> World {
        let mut w = World {
            frame: 0,
            players: [Player::default(); MAX_PLAYERS],
            phase: Phase::Fighting,
            effects: [None; MAX_EFFECTS],
            ground: crate::ground::NONE,
            ground_held: 0,
            bolts: [None; MAX_BOLTS],
            debris: [None; MAX_DEBRIS],
            gusts: [None; MAX_GUSTS],
            monsters: [None; MAX_MONSTERS],
            arena: ArenaId::PROVING_GROUND,
            critters: Critters::NONE,
            pack: None,
            lore: Lore::NONE,
            course: [crate::course::Run::default(); MAX_PLAYERS],
            paused: false,
            seats: MAX_PLAYERS as u8,
            valley: crate::valley::Journey::default(),
            away: Fighters::EMPTY,
            ready: Fighters::EMPTY,
            crowd: crate::crowd::Field::NONE,
            loose: [None; crate::keep::LOOSE],
            picked: [None; MAX_PLAYERS],
            hill: crate::versus::Koth::NONE,
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
    pub fn hunt<const N: usize>(classes: [Class; N]) -> World {
        World::hunt_of(classes, SpeciesId::RIDGEBACK)
    }

    /// A hunt against one creature of any species.
    pub fn hunt_of<const N: usize>(classes: [Class; N], species: SpeciesId) -> World {
        let mut herd = [None; MAX_MONSTERS];
        // As many of it as its species says a hunt holds: two for the Pair,
        // one for everybody else (`FightDecl::bodies`).
        let bodies = (species.get().fight.bodies as usize).clamp(1, MAX_MONSTERS);
        for slot in herd.iter_mut().take(bodies) {
            *slot = Some(species);
        }
        World::hunt_with(classes, herd)
    }

    /// A hunt against whichever creatures are named, one per slot, in the
    /// first one's arena ([`arena::for_species`]).
    pub fn hunt_with<const N: usize>(
        classes: [Class; N],
        herd: [Option<SpeciesId>; MAX_MONSTERS],
    ) -> World {
        let place = herd
            .iter()
            .flatten()
            .next()
            .map_or(ArenaId::PROVING_GROUND, |s| arena::for_species(*s).id);
        World::hunt_in(classes, herd, place)
    }

    /// A hunt against whichever creatures are named, in a chosen arena.
    pub fn hunt_in<const N: usize>(
        classes: [Class; N],
        herd: [Option<SpeciesId>; MAX_MONSTERS],
        place: ArenaId,
    ) -> World {
        World::hunt_all(World::seats_of(classes), herd, place).seated(N as u8)
    }

    /// [`World::hunt_in`] with every seat's class named.
    fn hunt_all(
        classes: [Class; MAX_PLAYERS],
        herd: [Option<SpeciesId>; MAX_MONSTERS],
        place: ArenaId,
    ) -> World {
        let mut w = World::with_all(classes);
        w.arena = arena::lookup(place).map_or(ArenaId::PROVING_GROUND, |a| a.id);
        // The lore is laid out by the first creature named: the fight's.
        if let Some(first) = herd.iter().flatten().next() {
            w.lore = Lore::of(*first);
        }
        for (slot, species) in herd.iter().enumerate() {
            let Some(id) = species else { continue };
            let sp = id.get();
            // A body, if it has one; a pack, if it brings one. The first pack
            // named is the fight's -- one per world -- and a body that brings
            // it owns it.
            if sp.has_body() {
                w.monsters[slot] = Some(Monster::new(*id));
            }
            if sp.pack.is_some() && w.pack.is_none() {
                let mut pack = Pack::new(*id, V3::ZERO);
                if sp.has_body() {
                    pack.owner = slot as u8;
                }
                w.pack = Some(pack);
            }
        }
        w.reset_positions();
        // **A place of the valley that is not a creature's room is the
        // valley**: Hearth, the Ring, a reach. A room is only the valley when
        // a seam leads into it (`World::arrive`), so `--hunt` and a room's
        // arena on its own are the fights they always were.
        if herd.iter().all(Option::is_none)
            && crate::valley::place(w.arena)
                .is_some_and(|p| !matches!(p.kind, crate::valley::Kind::Room(_)))
        {
            w.valley.on = true;
            // The map is composed here, as the world is built, so no frame
            // ever builds it: a frame does not allocate.
            crate::atlas::valley();
            // The marks were stood on the place's own floor; on the map the
            // floor is the land, metres up in a reach.
            let ground = w.terrain();
            for p in w.players.iter_mut() {
                let up = V3::new(p.pos.x, Fx::from_int(1000), p.pos.z);
                p.pos.y = ground.floor_below(up);
                if let Mechanic::Shadow(_) = p.mechanic {
                    p.mechanic = Mechanic::Shadow(class::Shadow::attending(p.pos, p.facing));
                }
            }
        }
        w
    }

    /// **Arrive in a place of the valley**: the world a seam leads to. A
    /// room starts its creature's hunt; anywhere else is the place as it
    /// stands. `at` is the seam of `to` you come in by, and where you stand;
    /// `None` is the place's own start. The journey comes along -- what has
    /// been beaten -- and everything about the last place does not.
    pub fn arrive<const N: usize>(
        classes: [Class; N],
        journey: crate::valley::Journey,
        to: ArenaId,
        at: Option<u8>,
        seats: u8,
    ) -> World {
        World::arrive_all(World::seats_of(classes), journey, to, at, seats)
    }

    /// [`World::arrive`] with every seat's class named.
    fn arrive_all(
        classes: [Class; MAX_PLAYERS],
        journey: crate::valley::Journey,
        to: ArenaId,
        at: Option<u8>,
        seats: u8,
    ) -> World {
        let place = crate::valley::place(to);
        let range = crate::valley::ranges::of(to);
        let mut w = match place.map(|p| p.kind) {
            Some(crate::valley::Kind::Room(species)) if range.is_none() => {
                World::hunt_all(classes, crate::replay::herd_of(species), to)
            }
            _ => World::hunt_all(classes, [None; MAX_MONSTERS], to),
        };
        crate::atlas::valley();
        w.valley = crate::valley::Journey {
            on: true,
            beaten: journey.beaten,
            keep: journey.keep,
            came: at.map_or(crate::valley::NONE, |k| k + 1),
            ..crate::valley::Journey::default()
        };
        if let Some(seam) = at.and_then(|k| place.and_then(|p| p.seam(k))) {
            // On the map, the ground under a mark is the land's.
            let here = w.terrain();
            for (i, p) in w.players.iter_mut().enumerate() {
                let mark = Mark::seat(&seam.marks, i);
                let up = V3::new(mark.at.x, Fx::from_int(1000), mark.at.z);
                p.pos = V3::new(mark.at.x, here.floor_below(up), mark.at.z);
                p.facing = mark.facing;
                p.vel = V3::ZERO;
                if let Mechanic::Shadow(_) = p.mechanic {
                    p.mechanic = Mechanic::Shadow(class::Shadow::attending(p.pos, p.facing));
                }
            }
        }
        let mut w = w.seated(seats);
        // A room arrived in is its hunt's home (exploration 0023).
        if w.hunting() {
            w.valley.den = Some(to);
        }
        // **Arriving on a range** wakes it as walking into it does, unless
        // it is resting: its herd, its temper, its temperament.
        w.valley.rested = journey.rested;
        if let Some(r) = range
            && !w.resting(to)
        {
            w.begin_hunt(r.species);
        }
        w
    }

    /// A versus match in a chosen arena.
    pub fn versus_in<const N: usize>(classes: [Class; N], place: ArenaId) -> World {
        World::hunt_in(classes, [None; MAX_MONSTERS], place)
    }

    /// The same fight from the top: these classes, the same creatures, the
    /// same arena, the same temper. What a reset or a class change starts.
    /// As many classes as are named; a seat past them takes the first.
    pub fn restarted<const N: usize>(&self, classes: [Class; N]) -> World {
        let classes = World::seats_of(classes);
        if self.valley.on {
            // In the valley a restart is this place again, from the seam you
            // came in by, with the journey kept.
            return World::arrive_all(
                classes,
                self.valley,
                self.arena,
                self.valley.came.checked_sub(1),
                self.seats,
            );
        }
        World::hunt_all(classes, self.hunted(), self.arena)
            .tempered(self.temper())
            .seated(self.seats)
    }

    /// The same fight with every creature in it at temper `n` -- its bodies
    /// and its pack alike (`crate::temper`). Clamped to the highest there is.
    /// A versus match has nothing to temper and comes back unchanged, which
    /// is the whole of why a temper cannot reach versus.
    ///
    /// Part of the hunt's **initial state**: called as the world is built,
    /// before its first frame, by whatever builds it -- the picker's travel
    /// on the wire, or a restart -- so both peers start the same fight.
    pub fn tempered(mut self, n: u8) -> World {
        let n = n.min(crate::temper::HIGHEST);
        for m in self.monsters.iter_mut().flatten() {
            m.temper = n;
        }
        if let Some(p) = self.pack.as_mut() {
            p.temper = n;
        }
        self
    }

    /// **Seat `seats` fighters**, one to [`MAX_PLAYERS`], and take the rest
    /// out of the world: a fighter nobody drives is a target the creature will
    /// happily cross the arena for, which is what the first long run of the
    /// harness measured. Versus keeps at least two bodies whatever is asked
    /// -- one seat is somebody training against the dummy -- so a duel is
    /// still a duel ([`World::present`]). Like [`World::tempered`], part of a
    /// fight's initial state, applied as the world is built.
    pub fn seated(mut self, seats: u8) -> World {
        self.seats = seats.clamp(1, MAX_PLAYERS as u8);
        self.park_absent();
        self
    }

    /// **How many bodies are in the fight**: the seats, and in versus never
    /// fewer than two. Every fighter past these is parked out of the world
    /// ([`World::park_absent`]) and plays no part in anything.
    pub fn present(&self) -> usize {
        let seats = (self.seats as usize).clamp(1, MAX_PLAYERS);
        if self.hunting() || self.valley.on && self.peaceful() {
            seats
        } else {
            seats.max(2)
        }
    }

    /// Is fighter `i` in the fight at all? Not if their seat is empty, and
    /// not if their player has left ([`World::away`]).
    pub fn is_present(&self, i: usize) -> bool {
        i < self.present() && !self.away.has(i)
    }

    /// **Has everybody in the fight pressed "Start match"?** Two at least:
    /// one alone is not a match.
    pub fn all_ready(&self) -> bool {
        let mut players = 0;
        for i in 0..MAX_PLAYERS {
            if self.is_present(i) {
                if !self.ready.has(i) {
                    return false;
                }
                players += 1;
            }
        }
        players >= 2
    }

    /// **The match everybody was ready for**: a fresh round of the versus
    /// stage the fight is on, or the first floating duel stage from anywhere
    /// else -- the hub, a hunt, a course. Its frame is this one, as a trip's.
    fn match_start(&self, classes: [Class; MAX_PLAYERS]) -> World {
        let place = match crate::versus::of(self.arena) {
            Some(_) => self.arena,
            None => crate::versus::STAGES[0].arena,
        };
        World {
            frame: self.frame,
            paused: self.paused,
            ..World::versus_in(classes, place).seated(self.seats)
        }
    }

    /// **Which seat a newcomer takes**: a free one on the smaller side
    /// ([`crate::versus::side`]), or on the side `coin` says when the two
    /// are level, or any free seat if that side has none. `None` when every
    /// seat is somebody's. Asked by whoever seats them (the room's host);
    /// [`World::seat_join`] is what every machine then does with the answer.
    pub fn open_seat(&self, coin: bool) -> Option<usize> {
        let mut on = [0usize; crate::versus::SIDES];
        for i in 0..MAX_PLAYERS {
            if self.is_present(i) {
                on[crate::versus::side(i) as usize] += 1;
            }
        }
        let side = if on[0] == on[1] {
            coin as u8
        } else {
            (on[1] < on[0]) as u8
        };
        let free = |i: &usize| !self.is_present(*i);
        (0..MAX_PLAYERS)
            .filter(free)
            .find(|&i| crate::versus::side(i) == side)
            .or_else(|| (0..MAX_PLAYERS).find(free))
    }

    /// **Somebody joins the fight in progress**, at seat `seat` as `class`:
    /// their fighter stands where a fresh start of this same fight would
    /// stand it -- built, and that one fighter taken from it -- and nobody
    /// else is touched. Applied to the same world by every machine of a
    /// match as its seats change (`net`'s lineup), so they agree.
    pub fn seat_join(&mut self, seat: usize, class: Class) {
        if seat >= MAX_PLAYERS {
            return;
        }
        let seats = self.seats.max(seat as u8 + 1);
        let mut classes = self.players.map(|p| p.class);
        classes[seat] = class;
        let mut grown = self.clone();
        grown.seats = seats;
        grown.away.unset(seat);
        let fresh = grown.restarted(classes);
        self.seats = seats;
        self.away.unset(seat);
        self.ready.unset(seat);
        self.players[seat] = fresh.players[seat];
    }

    /// **Put every fighter past [`World::present`] out of the world**: dead,
    /// still, and under the middle of the place's floor, where nothing reaches
    /// it and nothing draws it in view. What the valley has always done with
    /// a fighter nobody plays, done for every fight now there are four seats.
    fn park_absent(&mut self) {
        let present = self.present();
        if present >= MAX_PLAYERS && !self.away.any() {
            return;
        }
        // Under the hunt's home, wherever the world's frame is: the same
        // place whichever coordinates it is written in (exploration 0023).
        let den = self.den();
        let b = den.home;
        let under = V3::new(
            crate::math::midpoint(b.lo_x, b.hi_x),
            den.off.y.add(Fx::from_int(-1000)),
            crate::math::midpoint(b.lo_z, b.hi_z),
        );
        let away = self.away;
        for (i, p) in self.players.iter_mut().enumerate() {
            if i < present && !away.has(i) {
                continue;
            }
            p.pos = under;
            p.vel = V3::ZERO;
            p.health = 0;
            p.mount = monster::NO_MOUNT;
        }
    }

    /// The hunt's temper: its first creature's, or its pack's. Zero in versus.
    pub fn temper(&self) -> u8 {
        self.monster()
            .map(|m| m.temper)
            .or(self.pack.map(|p| p.temper))
            .unwrap_or(0)
    }

    /// **The hunt just won, if it was**: the creatures it was against, and the
    /// temper they were at. `Some` from the frame the last of them falls (or a
    /// defended thing wins it, bestiary P7) until the round's pause runs out,
    /// and `None` in versus, in a hunt still going, and in one lost.
    ///
    /// What a trophy is written from (world W1). The record itself is the
    /// player's and not the fight's, so it is kept outside the snapshot by the
    /// game; this is only the question it asks.
    pub fn hunt_won(&self) -> Option<([Option<SpeciesId>; MAX_MONSTERS], u8)> {
        match self.phase {
            Phase::RoundOver { winner, .. } if self.hunting() && winner != QUARRY => {
                Some((self.hunted(), self.temper()))
            }
            _ => None,
        }
    }

    /// The creatures this fight is against, one per slot, as `hunt_with`
    /// takes them: the bodies in their slots, and a pack that has no body in
    /// the first slot nobody holds.
    pub fn hunted(&self) -> [Option<SpeciesId>; MAX_MONSTERS] {
        let mut herd = self.monsters.map(|m| m.map(|m| m.species));
        if let Some(pack) = self.pack.filter(|p| p.owner == pack::NONE) {
            if let Some(free) = herd.iter_mut().find(|s| s.is_none()) {
                *free = Some(pack.species);
            }
        }
        herd
    }

    /// Where the fight is.
    pub fn arena(&self) -> &'static Arena {
        self.arena.get()
    }

    /// **The ground as it stands this frame**: the arena, the solids the fight
    /// has raised on it (slag, a wall, a cart), and its floor hazards placed in
    /// the world. What every body, ray and glance in the frame reads; see
    /// [`arena::Terrain`].
    pub fn terrain(&self) -> Terrain {
        // In the valley the place is on a map, and the map is around it.
        let mut ground = if self.valley.on {
            let mut g = self.valley_ground();
            g.den = self.den();
            g
        } else {
            Terrain::bare(self.arena.get())
        };
        if self.lore.owner.is_none() {
            return ground;
        }
        ground.floor = Floor::build(&self.lore, &self.monsters);
        let (raised, n) = ground.floor.solids::<{ arena::MAX_RAISED }>();
        for s in &raised[..n] {
            ground.raise(*s);
        }
        for o in objective::standing(&self.lore, self.arena.get()) {
            if o.decl.solid && !o.state.broken {
                ground.raise(o.solid());
            }
        }
        ground
    }

    /// Go somewhere else: the picker's request, carried on the wire as
    /// [`Input::travel`]. A fresh fight with the same classes, keeping only the
    /// frame number -- which the rollback session owns and checks.
    ///
    /// A request for a creature that is not registered is no request at all:
    /// the picker never sends one, and a peer on an older build that did would
    /// otherwise be sent somewhere the other cannot follow.
    ///
    /// `classes` are the fighters' classes after this frame's class changes,
    /// which a trip on the same frame keeps. A class change itself is not a
    /// trip: see [`World::recast`].
    fn travelled(&self, to: Travel, classes: [Class; MAX_PLAYERS]) -> Option<World> {
        let world = match to.destination()? {
            Destination::Class { .. }
            | Destination::Pause
            | Destination::Step
            | Destination::Left
            | Destination::Ready
            | Destination::Admit { .. }
            | Destination::Credit(_)
            | Destination::Feed(_)
            | Destination::Grant(_)
            | Destination::Cairn(_) => return None,
            Destination::Valley => {
                World::arrive_all(classes, self.valley, crate::valley::START, None, self.seats)
            }
            Destination::Restart => self.restarted(classes),
            Destination::Versus => World::with_classes(classes),
            Destination::Hunt(species, temper) => {
                crate::species::lookup(species)?;
                World::hunt_of(classes, species).tempered(temper)
            }
            // A course, or any arena: an unregistered one is no trip, for
            // the reason an unregistered creature is not.
            Destination::Arena(place) => {
                arena::lookup(place)?;
                World::versus_in(classes, place)
            }
        };
        Some(World {
            frame: self.frame,
            paused: self.paused,
            ..world.seated(self.seats)
        })
    }

    /// This frame's class changes, from both sides: each [`Destination::Class`]
    /// on the wire sets that seat's class, player one's first. `None` if there
    /// were none.
    fn recast(&self, wire: &[Input; MAX_PLAYERS]) -> Option<[Class; MAX_PLAYERS]> {
        let mut classes = self.players.map(|p| p.class);
        let mut any = false;
        for input in wire {
            if let Some(Destination::Class { seat, class }) = input.travel.destination()
                && seat < MAX_PLAYERS
            {
                classes[seat] = class;
                any = true;
            }
        }
        any.then_some(classes)
    }

    /// **One frame of a jump course** (`crate::course`): for each fighter,
    /// has she fallen into the pit, left the start, reached a gate? A fall
    /// stands her on the last gate she reached as a round would -- fresh,
    /// with whatever she had out taken back -- and keeps the clock running.
    /// A no-op in an arena that is not a course.
    fn step_course(&mut self) {
        let Some(course) = crate::course::of(self.arena) else {
            return;
        };
        for i in 0..MAX_PLAYERS {
            let event =
                crate::course::step(course, &mut self.course[i], self.players[i].pos, self.frame);
            if let crate::course::Event::Fell(gate) = event {
                let gate = gate.min(course.finish());
                let (stand, facing) = (course.gate(gate).stand(), course.facing(gate));
                let p = &mut self.players[i];
                let (wins, class) = (p.rounds_won, p.class);
                *p = Player {
                    pos: stand,
                    facing,
                    rounds_won: wins,
                    ..Player::new(class)
                };
                if let Mechanic::Shadow(_) = p.mechanic {
                    p.mechanic = Mechanic::Shadow(class::Shadow::attending(p.pos, p.facing));
                }
            }
        }
    }

    /// **One frame of the valley** (`crate::valley`). Where it is peaceful, a
    /// fighter who has died is stood on their last cairn -- or where they
    /// came in -- and one standing on a cairn is rested. Then the frame
    /// follows the fighters: into the place the first of them is in, or a
    /// room somebody has walked into, whose hunt starts
    /// (`crate::valley::open`). A no-op outside the valley.
    ///
    /// Nothing here replaces the world any more: the valley is one map, and
    /// walking is how you get anywhere in it.
    fn step_valley(&mut self) -> Option<World> {
        if !self.valley.on {
            return None;
        }
        let place = crate::valley::place(self.arena)?;
        let seats = (self.seats as usize).clamp(1, MAX_PLAYERS);
        let peaceful = self.peaceful();
        if peaceful {
            // **A fighter nobody plays is not in the valley**: kept under the
            // middle of the place's floor, where nothing reaches it and
            // nothing draws it in view -- rather than a body lying at every
            // seam you come through. The Ring keeps both (player two is the
            // dummy or the bot there), and a hunt takes an absent hunter out
            // as it always has.
            self.park_absent();
            for i in 0..seats {
                if self.players[i].health <= 0 {
                    self.stand_back(i, place);
                }
            }
        }
        // **A cairn is a checkpoint in a fight too**; it only rests you out
        // of one. The cairn on the Needle's top is on the roost, and stepping
        // onto the ruins wakes the Galewing: kept to peace, it never counted,
        // and a fall off the dome stood you back up a hundred metres down the
        // stair (2026-10-20, from play).
        let atlas = crate::atlas::valley();
        let origin = self.map_origin();
        for i in 0..seats {
            let feet = self.players[i].pos.add(origin);
            if let Some(k) = crate::valley::cairn_at(atlas, feet) {
                self.valley.cairn[i] = k as u16 + 1;
                if peaceful {
                    let p = &mut self.players[i];
                    p.health = p.health.max(t::health_of(p.class));
                }
            }
        }
        self.reframe();
        self.tend_range();
        self.step_keep();
        self.step_roam();
        None
    }

    /// The world a seam leads to, with this one's frame and pause.
    fn through(&self, seam: &crate::valley::Seam) -> World {
        let classes = self.players.map(|p| p.class);
        World {
            frame: self.frame,
            paused: self.paused,
            ..World::arrive_all(classes, self.valley, seam.to, Some(seam.at), self.seats)
        }
    }

    /// **A hunt in the valley, lost**: you wake outside, at the seam you went
    /// in by -- or in Hearth, for a room the world started in.
    fn woke_outside(&self) -> World {
        let mut w = self.woke_outside_as_was();
        // The creature took its egg back.
        for s in self.hunted().into_iter().flatten() {
            for i in 0..MAX_PLAYERS {
                w.valley.keep.lose_eggs(i, s);
            }
        }
        w
    }

    fn woke_outside_as_was(&self) -> World {
        let back = crate::valley::place(self.arena)
            .and_then(|p| self.valley.came.checked_sub(1).and_then(|k| p.seam(k)));
        match back {
            Some(seam) => self.through(seam),
            None => World {
                frame: self.frame,
                paused: self.paused,
                ..World::arrive(
                    self.players.map(|p| p.class),
                    self.valley,
                    crate::valley::START,
                    None,
                    self.seats,
                )
            },
        }
    }

    /// **A hunt in the valley, won**: the place goes quiet -- no creature,
    /// no pack, nothing on the floor -- and the hunters are rested where they
    /// stand, to walk out the way they came.
    fn quiet(&mut self) {
        self.home_gone();
        self.monsters = [None; MAX_MONSTERS];
        self.pack = None;
        self.critters = Critters::NONE;
        self.lore = Lore::NONE;
        self.effects = [None; MAX_EFFECTS];
        self.ground = crate::ground::NONE;
        self.ground_held = 0;
        self.bolts = [None; MAX_BOLTS];
        self.debris = [None; MAX_DEBRIS];
        self.gusts = [None; MAX_GUSTS];
        let seats = (self.seats as usize).clamp(1, MAX_PLAYERS);
        for p in self.players.iter_mut().take(seats) {
            let (pos, facing, wins, class) = (p.pos, p.facing, p.rounds_won, p.class);
            *p = Player {
                pos,
                facing,
                rounds_won: wins,
                ..Player::new(class)
            };
            if let Mechanic::Shadow(_) = p.mechanic {
                p.mechanic = Mechanic::Shadow(class::Shadow::attending(p.pos, p.facing));
            }
        }
    }

    /// Stand fighter `i` back up after a death in a peaceful place: on the
    /// last cairn they touched, or where they came in, fresh -- as a fall on
    /// a jump course stands you on your checkpoint.
    fn stand_back(&mut self, i: usize, place: &crate::valley::Place) {
        let here = self.terrain();
        let on_mark = |m: &arena::Mark| {
            let up = V3::new(m.at.x, Fx::from_int(1000), m.at.z);
            (V3::new(m.at.x, here.floor_below(up), m.at.z), m.facing)
        };
        // A cairn is a box of the map, wherever it is; it is stood on in this
        // place's coordinates.
        let origin = self.map_origin();
        let cairn = self.valley.cairn[i]
            .checked_sub(1)
            .and_then(|k| crate::atlas::valley().solids.get(k as usize))
            .map(|s| Solid {
                min: s.min.sub(origin),
                max: s.max.sub(origin),
                material: s.material,
                shape: s.shape,
            });
        let p = &mut self.players[i];
        let (pos, facing) = match cairn {
            Some(top) => (
                V3::new(
                    crate::math::midpoint(top.min.x, top.max.x),
                    top.max.y,
                    crate::math::midpoint(top.min.z, top.max.z),
                ),
                p.facing,
            ),
            None => match self.valley.came.checked_sub(1).and_then(|k| place.seam(k)) {
                Some(seam) => on_mark(&Mark::seat(&seam.marks, i)),
                None => on_mark(&Mark::seat(&here.spawns.versus, i)),
            },
        };
        let (wins, class) = (p.rounds_won, p.class);
        *p = Player {
            pos,
            facing,
            rounds_won: wins,
            ..Player::new(class)
        };
        if let Mechanic::Shadow(_) = p.mechanic {
            p.mechanic = Mechanic::Shadow(class::Shadow::attending(p.pos, p.facing));
        }
    }

    /// Is there anything to hunt? The one condition friendly fire, targets and
    /// the round's end all read: a creature's presence rather than a flag.
    pub fn hunting(&self) -> bool {
        self.monsters.iter().any(Option::is_some) || self.pack.is_some()
    }

    /// **Is this a peaceful place of the valley**: the town, a reach, a room
    /// whose creature is beaten -- anywhere in the valley but the Ring and a
    /// hunt still going. Nobody can hurt anybody, and a death is a cairn
    /// rather than a round (`crate::valley`).
    pub fn peaceful(&self) -> bool {
        self.valley.on
            && !self.hunting()
            && crate::valley::place(self.arena).is_none_or(|p| p.peaceful())
    }

    /// **May the fighters hurt each other?** In versus, yes; in a hunt and in
    /// the valley's peaceful places, no. The one question every blow between
    /// fighters asks.
    pub fn pvp(&self) -> bool {
        !self.hunting() && !self.peaceful()
    }

    /// **Fighter `i`'s own side**, as seat bits, leaving `i` out: who their
    /// shots pass through ([`aim::Targets::spare`]). Nobody but on a hill.
    pub fn allies(&self, i: usize) -> u8 {
        let mut bits = 0;
        for j in 0..MAX_PLAYERS {
            if j != i && self.hill.on && crate::versus::side(i) == crate::versus::side(j) {
                bits |= 1 << j;
            }
        }
        bits
    }

    /// **May fighter `a` hurt fighter `b`?** [`World::pvp`], never
    /// themselves, and on a King of the Hill map never somebody on their
    /// own side (`crate::versus::side`). A duel stage is everybody against
    /// everybody, as versus always was.
    pub fn foes(&self, a: usize, b: usize) -> bool {
        a != b && self.pvp() && !(self.hill.on && crate::versus::side(a) == crate::versus::side(b))
    }

    /// The first creature, for the code and tools that are about one: the
    /// harness, the HUD, most tests.
    pub fn monster(&self) -> Option<&Monster> {
        self.monsters.iter().flatten().next()
    }

    /// The first creature, to change it.
    pub fn monster_mut(&mut self) -> Option<&mut Monster> {
        self.monsters.iter_mut().flatten().next()
    }

    /// **One frame of King of the Hill** (`crate::versus`): the dead wait
    /// and stand back up at their side's start, the point counts who is on
    /// it, and a side whose clock has run out unchallenged wins the round.
    fn step_the_hill(&mut self) {
        let Some(hill) = crate::versus::of(self.arena).and_then(|s| s.hill) else {
            return;
        };
        for i in 0..MAX_PLAYERS {
            if !self.is_present(i) || self.players[i].health > 0 {
                self.hill.respawn[i] = 0;
                continue;
            }
            if self.hill.respawn[i] == 0 {
                self.hill.respawn[i] = t::hill_respawn().max(1);
                continue;
            }
            self.hill.respawn[i] -= 1;
            if self.hill.respawn[i] == 0 {
                self.stand_on_mark(i);
            }
        }
        let on: [bool; MAX_PLAYERS] = std::array::from_fn(|i| {
            let p = &self.players[i];
            self.is_present(i) && p.health > 0 && hill.holds(p.pos)
        });
        if let Some(side) = self.hill.step(on) {
            let present = self.present();
            for (i, p) in self.players.iter_mut().enumerate().take(present) {
                if crate::versus::side(i) == side {
                    p.rounds_won += 1;
                }
            }
            self.phase = Phase::RoundOver {
                winner: side,
                left: t::round_over_frames(),
            };
        }
    }

    /// Stand fighter `i` up fresh on their start mark, as a round does:
    /// what a respawn on the hill is.
    fn stand_on_mark(&mut self, i: usize) {
        let here = self.terrain();
        let mark = Mark::seat(&here.spawns.versus, i);
        let p = &mut self.players[i];
        let (wins, class) = (p.rounds_won, p.class);
        *p = Player {
            pos: V3::new(mark.at.x, here.ground_under(mark.at), mark.at.z),
            facing: mark.facing,
            rounds_won: wins,
            ..Player::new(class)
        };
        if let Mechanic::Shadow(_) = p.mechanic {
            p.mechanic = Mechanic::Shadow(class::Shadow::attending(p.pos, p.facing));
        }
    }

    /// Put both fighters back on their marks. Keeps round wins and classes.
    fn reset_positions(&mut self) {
        // Nothing in the air survives a round. A bolt still flying when the
        // last one ended would land on somebody standing on their mark.
        self.bolts = [None; MAX_BOLTS];
        // Nor does her ground: a round starts on clean floor.
        self.ground = crate::ground::NONE;
        self.ground_held = 0;
        self.debris = [None; MAX_DEBRIS];
        self.gusts = [None; MAX_GUSTS];
        // A course's runs start again with the round, clock and all.
        self.course = [crate::course::Run::default(); MAX_PLAYERS];
        // So does a game on the hill: locked, nobody's, both clocks full.
        self.hill = crate::versus::Koth::start(self.arena);
        let here = self.arena();
        // The hunt's lore starts again with the round: nothing on the floor,
        // nothing heard, every defended thing whole and back at its start.
        if let Some(owner) = self.lore.owner {
            self.lore = Lore::of(owner);
            objective::set_up(&mut self.lore, here);
        }
        for (i, p) in self.players.iter_mut().enumerate() {
            let wins = p.rounds_won;
            let class = p.class;
            let mark = Mark::seat(&here.spawns.versus, i);
            *p = Player {
                // **On the ground under the mark**: the floor everywhere a
                // mark has stood, the plateau's top in the Cliffs.
                pos: V3::new(mark.at.x, here.ground_under(mark.at), mark.at.z),
                facing: mark.facing,
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
        let count = self.monsters.iter().flatten().count() as i32;
        let mut hunter_spawn = None;
        let ground = self.terrain();
        for (slot, place) in self.monsters.iter_mut().enumerate() {
            let Some(old) = *place else { continue };
            let mut beast = Monster::new(old.species);
            // Its temper is the hunt's, not its position's: kept.
            beast.temper = old.temper;
            let sp = beast.sp();
            // Well back, and facing the hunters. A creature that spawns on top
            // of you has taken the opening read away from both of you. Two
            // stand side by side, a keep-out apart either side of the line.
            match here.hunt_marks() {
                Some(marks) => {
                    let mark = marks.creatures[slot.min(1)];
                    beast.pos = V3::new(mark.at.x, Fx::ZERO, mark.at.z);
                    beast.yaw = crate::math::atan2_turns(mark.facing.z, mark.facing.x);
                }
                None => {
                    let spread = sp.margin().mul(Fx::from_int(slot as i32 * 2 - (count - 1)));
                    beast.pos = V3::new(sp.spawn(), Fx::ZERO, spread);
                    beast.yaw = HALF_TURN;
                }
            }
            beast.settle(&ground);
            beast.brain.seen = self.players[0].pos;
            // And it takes a moment to notice you: it stands its ground,
            // turns to face you, and throws nothing until the moment is spent
            // or somebody hits it. It opened with a spray and a charge that
            // landed before anybody had moved the camera. See
            // `Species::hunt_grace`.
            beast.brain.grace = sp.hunt_grace();
            hunter_spawn.get_or_insert(sp.hunter_spawn());
            *place = Some(beast);
        }
        // The pack, mustered where its species says: in a knot at its own
        // spawn distance, facing the hunters, or beside the monster that owns
        // it. Its home -- the den it runs for -- is where it mustered.
        if let Some(old) = self.pack {
            let sp = old.species.get();
            let owner = self
                .monsters
                .get(old.owner as usize)
                .and_then(|m| m.as_ref());
            let (home, facing) = match (owner, here.hunt_marks()) {
                (Some(beast), _) => (beast.pos, V3::from_turns(beast.yaw)),
                (None, Some(marks)) => {
                    let mark = marks.creatures[0];
                    (V3::new(mark.at.x, Fx::ZERO, mark.at.z), mark.facing)
                }
                (None, None) => (
                    V3::new(sp.pack_fx(pack::PackKnob::Spawn), Fx::ZERO, Fx::ZERO),
                    V3::new(Fx::ONE.neg(), Fx::ZERO, Fx::ZERO),
                ),
            };
            self.pack = pack::muster(old.species, home, facing, &mut self.critters).map(|mut p| {
                p.owner = old.owner;
                p
            });
            hunter_spawn.get_or_insert(sp.hunter_spawn());
        }
        if let Some(back) = hunter_spawn {
            for (i, p) in self.players.iter_mut().enumerate() {
                match here.hunt_marks() {
                    Some(marks) => {
                        let mark = Mark::seat(&marks.hunters, i);
                        p.pos = V3::new(mark.at.x, here.ground_under(mark.at), mark.at.z);
                        p.facing = mark.facing;
                    }
                    None => {
                        let x = back.neg();
                        // Two either side of the line, and the third and
                        // fourth a flank further out (`Mark::seat`).
                        let side = Fx::from_int(if i % 2 == 0 { -1 } else { 1 });
                        let flank = t::seat_flank().mul(Fx::from_int((i / 2) as i32));
                        let z = Fx::from_int(if i % 2 == 0 { -2 } else { 2 }).add(side.mul(flank));
                        p.pos = V3::new(x, here.ground_under(V3::new(x, GROUND_Y, z)), z);
                        p.facing = V3::new(Fx::ONE, Fx::ZERO, Fx::ZERO);
                    }
                }
            }
        }
    }

    /// Advance exactly one tick. This is the whole simulation.
    ///
    /// Must stay a pure function of `(self, inputs)`: no clocks, no randomness
    /// that is not seeded from state, no iteration over unordered collections.
    /// **One frame**, from an input per seat: as many as the caller has, one
    /// to [`MAX_PLAYERS`]; the seats past those send nothing. A fighter not
    /// [present](World::present) is put back out of the world before and
    /// after, so nothing in the frame -- a push, a fall, a respawn -- can
    /// bring it in.
    pub fn advance<const N: usize>(&mut self, wire: [Input; N]) {
        self.advance_with(wire, &[]);
    }

    /// [`World::advance`], **fed this frame's due aggregates** of the crowd
    /// field (`crate::crowd`): exactly those whose `Aggregate::due` is the
    /// frame being made, in region order, from the session layer's tape of
    /// them. A rollback re-feeds them the way it re-feeds inputs.
    pub fn advance_with<const N: usize>(
        &mut self,
        wire: [Input; N],
        due: &[crate::crowd::Aggregate],
    ) {
        const { assert!(N >= 1 && N <= MAX_PLAYERS, "one to four inputs") };
        let mut all = [Input::default(); MAX_PLAYERS];
        all[..N].copy_from_slice(&wire);
        self.follow_the_crowd();
        self.crowd.feed(due);
        // Nobody in a seat, nothing from it: not a press, not a trip. A seat
        // whose player has left is the same, for as long as it is said.
        self.away = Fighters::EMPTY;
        for (i, input) in all.iter().enumerate() {
            if input.travel.destination() == Some(Destination::Left) {
                self.away.set(i);
            }
        }
        let present = self.present();
        for (i, input) in all.iter_mut().enumerate() {
            if i >= present || self.away.has(i) {
                *input = Input::default();
            }
        }
        self.park_absent();
        // A fresh fight built inside the step -- a trip, a restart, waking
        // outside -- is built knowing nothing of who has gone: they are
        // still gone in it. A seat admitted this frame has come, not gone:
        // it had nobody's input until now.
        let mut away = self.away;
        for input in &all {
            if let Some(Destination::Admit { seat, .. }) = input.travel.destination() {
                away.unset(seat);
            }
        }
        self.step(all);
        self.away = away;
        self.park_absent();
    }

    /// **Which regions the field follows**: the neighbours of every region
    /// a present fighter stands in, as of this frame -- a function of the
    /// world, so every peer follows the same streams. A region followed
    /// last frame and not this one is forgotten.
    fn follow_the_crowd(&mut self) {
        let grid = crate::region::Grid::tuned();
        let origin = self.map_origin();
        let mut want = [None; crate::crowd::MAX_STREAMS];
        let mut n = 0;
        for i in 0..self.present() {
            if !self.is_present(i) {
                continue;
            }
            let home = grid.home(self.players[i].pos.add(origin));
            for r in home.neighbours() {
                if !want[..n].contains(&Some(r)) && n < want.len() {
                    want[n] = Some(r);
                    n += 1;
                }
            }
        }
        for s in self.crowd.streams.iter_mut() {
            if s.on && !want[..n].contains(&Some(s.region)) {
                *s = crate::crowd::Stream::NONE;
            }
        }
        for r in want[..n].iter().flatten() {
            self.crowd.follow(*r);
        }
    }

    fn step(&mut self, wire: [Input; MAX_PLAYERS]) {
        self.frame = self.frame.wrapping_add(1);
        self.picked = [None; MAX_PLAYERS];

        // A creature credited to the valley's journey from somebody's
        // trophies (`Destination::Credit`): a bit, on the frame it arrives,
        // before anything this frame reads the waystones.
        if self.valley.on {
            for input in &wire {
                if let Some(Destination::Credit(species)) = input.travel.destination() {
                    self.valley.credit(species);
                }
            }
            // An item used from a bag, or a dev's grant (`crate::keep`),
            // seat by seat.
            for (i, input) in wire.iter().enumerate() {
                match input.travel.destination() {
                    Some(Destination::Feed(item)) => self.fed(i, item),
                    Some(Destination::Grant(item)) => self.granted(i, item),
                    _ => {}
                }
            }
            // **Fast travel**, out of a fight: everybody on the cairn,
            // stood up on it as after a fall, and the frame follows them
            // there this frame (`reframe`). The last seat to ask wins.
            let to = wire
                .iter()
                .rev()
                .find_map(|input| match input.travel.destination() {
                    Some(Destination::Cairn(k)) => Some(k),
                    _ => None,
                });
            if let Some(k) = to
                && self.peaceful()
                && let Some(&box_) = crate::valley::cairns().get(k as usize)
                && let Some(place) = crate::valley::place(self.arena)
            {
                let seats = (self.seats as usize).clamp(1, MAX_PLAYERS);
                for i in 0..seats {
                    self.valley.cairn[i] = box_ as u16 + 1;
                    self.stand_back(i, place);
                }
            }
        }

        // The picker. A request from either side is a fresh fight, and the
        // frame it arrives on is spent building it: player one's request wins
        // if both send one on the same frame, which both machines agree on
        // because both have both inputs.
        //
        // A class change is the same kind of thing (Tab, and the pickers): the
        // same fight restarted with the new classes, on the frame it arrives.
        // Both players may ask on one frame and both are honoured, and a trip
        // on that frame keeps them.
        let recast = self.recast(&wire);
        let classes = recast.unwrap_or_else(|| self.players.map(|p| p.class));
        // A newcomer seated by the room's host, before anything else this
        // frame reads who is in the fight.
        for input in &wire {
            if let Some(Destination::Admit { seat, class }) = input.travel.destination() {
                self.seat_join(seat, class);
            }
        }
        // "Start match", from anybody in the fight: their flag flips. A seat
        // not in the fight has none.
        for (i, input) in wire.iter().enumerate() {
            if input.travel.destination() == Some(Destination::Ready) && self.is_present(i) {
                if self.ready.has(i) {
                    self.ready.unset(i);
                } else {
                    self.ready.set(i);
                }
            }
        }
        for i in 0..MAX_PLAYERS {
            if !self.is_present(i) {
                self.ready.unset(i);
            }
        }
        let next = wire
            .iter()
            .find_map(|i| self.travelled(i.travel, classes))
            .or_else(|| {
                recast.map(|classes| World {
                    frame: self.frame,
                    paused: self.paused,
                    ..self.restarted(classes).seated(self.seats)
                })
            })
            .or_else(|| self.all_ready().then(|| self.match_start(classes)));
        // Pause and step, from either side: one press is one toggle, and a
        // step pauses as well as playing its frame.
        let asked = |d: Destination| wire.iter().any(|i| i.travel.destination() == Some(d));
        let step = asked(Destination::Step);
        let toggle = asked(Destination::Pause);
        if let Some(next) = next {
            *self = next;
            self.paused = (self.paused != toggle) || step;
            return;
        }
        self.paused = (self.paused != toggle) || step;
        if self.paused && !step {
            return;
        }

        let wire = self.steer_the_balls(wire);
        let here = self.terrain();
        let here = &here;
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
                let present = self.present();
                for p in self.players.iter_mut().take(present) {
                    settle(p, here);
                    advance_clocks(p);
                    step_aloft(p, &field, here);
                }
                return;
            }
            // **A hunt in the valley does not start again.** Won, the place
            // goes quiet and you walk out the way you came; lost, you wake
            // outside, at the seam you went in by.
            if self.valley.on && self.hunting() {
                // Its home's, wherever the hunt has got to (exploration 0023).
                let den = self.den_id();
                let range = crate::valley::ranges::index(den);
                if self.hunt_won().is_some() {
                    self.quiet();
                    self.phase = Phase::Fighting;
                    // A range beaten rests a while before it is back.
                    if let Some(k) = range {
                        self.valley.rested[k] = self.frame.max(1);
                    }
                } else if range.is_some() || den != self.arena {
                    // **Lost on a range**, or far from the room it began in:
                    // nothing to wake outside of. The creature goes back to
                    // its grazing, it took its egg back, and everybody stands
                    // up on their last cairn, as a death in a reach does.
                    for s in self.hunted().into_iter().flatten() {
                        for i in 0..MAX_PLAYERS {
                            self.valley.keep.lose_eggs(i, s);
                        }
                        for l in self.loose.iter_mut() {
                            if l.is_some_and(|l| l.item == crate::keep::Item::Egg(s)) {
                                *l = None;
                            }
                        }
                    }
                    self.calm();
                    self.valley.roused = false;
                    self.valley.stock = 0;
                    if let Some(place) = crate::valley::place(self.arena) {
                        let seats = (self.seats as usize).clamp(1, MAX_PLAYERS);
                        for i in 0..seats {
                            self.stand_back(i, place);
                        }
                    }
                } else {
                    *self = self.woke_outside();
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
        // **The earth jump's stone keeps under her while she holds forward.**
        // Nothing in the air slows a body down, so a stone that only left a
        // little slower would never be a choice: held forward, it keeps pace
        // under her and she comes down on it; let go, or strafe, and it goes
        // on at its own slower speed and drops behind -- a stone in the air to
        // Gale back down at somebody. See `Structure::aloft`.
        for (i, input) in wire.iter().enumerate() {
            let p = self.players[i];
            if p.class == Class::Elementalist && !p.grounded && input.move_axis() == (0, 1) {
                stones::keep_under(&mut self.players, i);
            }
            // Stood on, an aloft stone is a platform rather than a thing in
            // flight: it stops going anywhere but its slow way down, so it
            // does not slide out from under whoever is on it.
            if p.grounded {
                if let Some(index) = stones::under(&self.players, p.pos) {
                    stones::bear(&mut self.players, index);
                }
            }
        }
        let landed = stones::step(&mut self.players, here);
        for (index, at) in landed.iter().enumerate() {
            if let Some(at) = *at {
                debris::shatter(
                    &mut self.debris,
                    (index / crate::class::MAX_STRUCTURES) as u8,
                    at,
                );
            }
        }
        let field = stones::gather(&self.players);

        // The creature decides and moves first, so that riders are carried by
        // a transform that is already final for this frame. It is handed a
        // deliberately small window on the fighters -- positions and
        // velocities, no buttons -- so "it reads your inputs" is not a thing
        // that can quietly become true. See `monster::Quarry`.
        let mut spin = [Fx::ZERO; MAX_MONSTERS];
        let senses = self.senses(here, &field);
        let herd_before = self.monsters;
        for (slot, place) in self.monsters.iter_mut().enumerate() {
            let Some(mut beast) = *place else { continue };
            let before = beast.yaw;
            let seen: [Quarry; MAX_PLAYERS] = std::array::from_fn(|i| {
                let real = Quarry {
                    pos: self.players[i].pos,
                    vel: self.players[i].vel,
                    alive: self.players[i].health > 0,
                    // Somebody it has swallowed is not standing on it.
                    aboard: self.players[i].aboard() && !inside(&self.players[i], &herd_before),
                    stunned: self.players[i].action.stunned(),
                };
                decoy_for(&self.effects, i, &beast, real)
            });
            beast.step_with(&seen, &senses[slot], here, &self.lore);
            // Its *heading*, not its wobble: a shake would otherwise spin the
            // rider's camera as hard as it spins the animal, and you are about
            // to be thrown off anyway.
            spin[slot] = crate::math::wrap_turns(beast.yaw.sub(before));
            *place = Some(beast);
        }

        // The pack, after the creatures for the same reason riders are: a
        // critter standing on one is carried by a pose that is final for this
        // frame. It sees the fighters' bodies and nothing of their buttons --
        // see `pack::Seen`.
        if let Some(mut brain) = self.pack {
            let seen = self.players;
            let beasts = self.monsters;
            let effects = self.effects;
            let nobody = Critters::NONE;
            let scene = Scene {
                stones: &field,
                players: &seen,
                effects: &effects,
                quarry: &beasts,
                critters: &nobody,
                arena: here,
            };
            // The crowd next door, by the field: what a calm pack hears.
            let heard = self
                .crowd
                .crowd_around(crate::region::Grid::tuned().home(brain.home.add(self.map_origin())));
            pack::step(
                &mut brain,
                &mut self.critters,
                &pack::World {
                    players: &seen,
                    herd: &beasts,
                    scene: &scene,
                    frame: self.frame,
                    heard,
                },
            );
            self.pack = Some(brain);
        }

        // **The species' own rules**, after its bodies have moved and before
        // anybody else does: where its tar spreads, when a vent blows. Then
        // the ground is placed again -- a vent rides the plate that just moved,
        // and anything the species put down this frame is on the floor for the
        // fighters' step below -- and the floor does what it does to whoever
        // is standing in it.
        if let Some(hook) = self.lore.owner.and_then(|s| s.get().fight.frame) {
            hook(self);
        }
        let ground = if self.lore.owner.is_some() {
            self.terrain()
        } else {
            *here
        };
        let here = &ground;
        self.feel_the_floor(here);

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
                let spin = spin[monster::mount_slot(p.mount).min(MAX_MONSTERS - 1)];
                p.carry_yaw = crate::math::wrap_turns(p.carry_yaw.add(spin));
            }
            thaw(p, wire[i].turned(p.carry_yaw))
        });
        // **A fighter held inside a creature** sends their keys to it first
        // (`FightDecl::from_inside`): what they may still do is its say.
        let mut inputs = inputs;
        for (i, input) in inputs.iter_mut().enumerate() {
            let p = self.players[i];
            if !inside(&p, &self.monsters) {
                continue;
            }
            let slot = monster::mount_slot(p.mount).min(MAX_MONSTERS - 1);
            if let Some(hook) = self.monsters[slot].and_then(|m| m.sp().fight.from_inside) {
                *input = hook(self, slot, i, *input);
            }
        }
        // Who is held in an impact freeze this frame. Worked out once, before
        // anybody moves, because three separate passes below have to agree
        // about it: the step, what a move does on its first active frame, and
        // the beams. A body that is frozen on the first frame of its hit would
        // otherwise fire its leap or its beam again on every frame of the
        // freeze, since nothing about its action moved.
        // A seat nobody is in is held still like a frozen body: it is out of
        // the world, and stepping it is a body falling under the land.
        let present = self.present();
        let frozen: [bool; MAX_PLAYERS] =
            std::array::from_fn(|i| i >= present || self.players[i].frozen > 0);
        // Who stands on her own ground, as the frame finds them: the input
        // below reads it (the swim, the strong version), so before it.
        self.mark_the_blood();

        // Everything the aiming ray can meet, as it stood at the top of the
        // frame. Copied rather than borrowed because the loop below takes each
        // fighter mutably -- and snapshotting is right anyway: both players
        // aim against the same world, so neither ordering wins.
        let beast = self.monsters;
        let crowd = self.critters;
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
            // Held still: no step, no clocks, not a centimetre of travel. The
            // knockback waiting in `vel` is spent when it ends.
            if frozen[i] {
                p.frozen = p.frozen.saturating_sub(1);
                continue;
            }
            let scene = Scene {
                stones: &field,
                players: &seen,
                effects: &effects,
                quarry: &beast,
                critters: &crowd,
                arena: here,
            };
            step_player(p, i, input, &field, &beast, &scene, carrying[i]);
            advance_clocks(p);
            step_aloft(p, &field, here);
            fade_grey(p, frame);
        }
        self.land_the_thornfall(&seen);
        // **Falls** (bestiary P6), and **noise** (P5): both read what the step
        // just did to each body, against the body as the frame found it.
        self.land(&seen);
        crate::noise::listen(
            &mut self.lore,
            &seen,
            &self.players,
            |p| here.material_under(p),
            frame,
        );
        // Landing with a Downdraft still blowing: the burst. Read here, after
        // the step that put her feet down and before anything else moves,
        // because the column is an effect and the body's step cannot see it.
        for i in 0..MAX_PLAYERS {
            if self.players[i].since_landed == 1 && self.players[i].air_frames > 0 {
                self.land_the_draft(i);
            }
        }
        // A dodge through a stone breaks it. Done here rather than in the
        // dodge branch because the branch only has her body, and the stone
        // and the ground it leaves are the world's.
        for i in 0..MAX_PLAYERS {
            let slot = self.players[i].breaking;
            if slot != NO_STONE {
                self.players[i].breaking = NO_STONE;
                self.break_through(i, slot as usize);
            }
        }
        // **Apotheosis** (2026-10-12): the frame a Dual mage's wings erupt,
        // a nova of both forces bursts from her. Read off the step that just
        // set her ascending against the body the frame found, because the
        // nova is an effect and the step only has her body.
        for (i, was) in seen.iter().enumerate() {
            if !dual::ascending(was) && dual::ascending(&self.players[i]) {
                self.apotheosis(i);
            }
        }
        // **The Flurry dash's disk breaks** (2026-10-13) the frame her run
        // reaches it: a burst where it stood, the size of the disk.
        for (i, was) in seen.iter().enumerate() {
            let now = &self.players[i];
            let running = |q: &Player| {
                q.class == Class::DualMage
                    && q.haul > 0
                    && matches!(
                        q.action,
                        Action::Recovery {
                            kind: moves::dual::FLURRY_DASH,
                            ..
                        }
                    )
            };
            if running(was) && !running(now) {
                let (_, disk) = moves::flurry_line(was.aim_path.from, was.aim_path.to);
                let mut burst = Effect::cast(
                    EffectKind::Burst,
                    i as u8,
                    was.class,
                    moves::dual::FLURRY_DASH,
                    disk,
                    V3::ZERO,
                    t::body_height().mul(Fx::ratio(1, 2)),
                );
                burst.home = was.pos;
                burst.power = was.thrown_at;
                spawn_effect(&mut self.effects, burst);
            }
        }
        // A blink **drinks** its pool (2026-10-12): the teleport is a heal and
        // a door at once, as walking into it would be -- her own blood is
        // still only a door. Done here rather than in the dodge branch
        // because the branch only has her body, and the pool is an effect.
        for i in 0..MAX_PLAYERS {
            let slot = self.players[i].blinked as usize;
            if slot < MAX_EFFECTS
                && let Some(pool) = self.effects[slot].filter(|e| e.is_a_pool())
            {
                self.players[i].absorb(pool.banked * t::stain_walk_drinks() / 100);
                // And she arrives in a burst of it: Thornfall's ring, at its
                // size.
                let m = moves::get(Class::BloodMage, moves::blood::THORNFALL);
                self.effects[slot] = Some(Effect::cast(
                    EffectKind::Nova,
                    i as u8,
                    Class::BloodMage,
                    moves::blood::THORNFALL,
                    pool.pos,
                    V3::ZERO,
                    m.radius,
                ));
            }
            self.players[i].blinked = NO_POOL;
        }

        // What a move does *as it comes out*, on its first active frame: the
        // leap of a leaping move, and whatever it leaves standing in the world.
        for (i, input) in inputs.into_iter().enumerate() {
            if frozen[i] {
                continue;
            }
            let p = self.players[i];
            // The Air ball is raised on the press, before the move has any
            // frames of its own: the hold is the ball growing where it sits.
            if let Action::Channel { kind, held: 0 } = p.action {
                if p.class == Class::Elementalist && kind == moves::elementalist::AIR_BALL {
                    self.raise_the_ball(i);
                }
                // The Bramble begins where she is aiming when the hold opens.
                if p.class == Class::BloodMage && kind == moves::blood::BRAMBLE {
                    self.lay_the_plague(i);
                }
            }
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
                // The Updraft from the floor is a takeoff, and it is worth
                // more than a jump: higher, and a push along her run.
                if p.class == Class::Elementalist
                    && kind == moves::elementalist::UPDRAFT
                    && p.grounded
                {
                    updraft_off_the_floor(&mut self.players[i], input);
                }
            }
            // `E` on the Reaver: the second body goes out, or comes home
            // through whatever is in the way. On the first active frame like
            // everything else a move does, so the startup is a window somebody
            // can punish rather than a formality.
            if p.class == Class::ShadowReaver && kind == SLOT_MECHANIC {
                self.order_the_shadow(i, p.aim_at());
            }
            // The Reaver's second pass (2026-10-10): what each move does on
            // the frame it comes out, besides its volume.
            if p.class == Class::ShadowReaver {
                self.reaver_comes_out(i, kind);
            }
            // The Champion's full charges (exploration 0015): the same.
            if p.class == Class::Champion {
                self.champion_comes_out(i, kind);
            }
            // The Bulwark on three clicks (2026-10-10): the same, for his.
            if p.class == Class::Bulwark {
                match kind {
                    moves::bulwark::REBOUND => self.rebound(i),
                    moves::bulwark::RAM => {
                        let ahead = V3::new(p.facing.x, Fx::ZERO, p.facing.z)
                            .normalized()
                            .scale(t::ram_speed());
                        let q = &mut self.players[i];
                        q.vel = V3::new(ahead.x, t::ram_lift(), ahead.z);
                        q.grounded = false;
                        q.air_stall = 0;
                    }
                    moves::bulwark::UNLOAD => {
                        let full = bulwark::fullness(&p);
                        let lift = t::unload_lift().add(t::unload_lift_full().mul(full));
                        let q = &mut self.players[i];
                        if let Mechanic::Shield(sh) = q.mechanic {
                            q.mechanic = Mechanic::Shield(sh.with_weight(Fx::ZERO));
                        }
                        q.vel.y = lift;
                        q.grounded = false;
                        q.air_stall = 0;
                    }
                    moves::bulwark::SHIELD_STEP => self.shield_step(i),
                    _ => {}
                }
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
                if let Some(at) =
                    aim::planted_ahead(p.pos, p.facing, t::landfall_ahead(), &field, here)
                {
                    stones::raise(
                        &mut self.players[i],
                        class::Structure::slammed(at, p.facing),
                    );
                }
            }
            // Fissure: the crack races from the stone she held churning. See
            // `race_the_crack`.
            if p.class == Class::Elementalist && kind == SLOT_COMMITTED {
                self.race_the_crack(i, input);
            }
            // The two drafts: a column of air on her own feet. Her own lift
            // is the move's `self_lift`, applied above like any leaping
            // move's; the column is what catches everybody and everything
            // else, and is what is drawn. The Downdraft drives her down as
            // well, which no row field says because none goes that way.
            if p.class == Class::Elementalist && kind == moves::elementalist::UPDRAFT {
                self.raise_the_draft(i, EffectKind::Updraft);
            }
            // The earth jump: after the ordinary lift above, which it makes
            // bigger off a stone that shatters under her, and with the stone
            // it brings up leaving at a share of the speed she now has.
            if p.class == Class::Elementalist && kind == moves::elementalist::EARTH_JUMP {
                self.earth_jump(i);
            }
            // The Blood mage's moves, as each comes out. See
            // `blood_comes_out`.
            if p.class == Class::BloodMage {
                self.blood_comes_out(i, kind);
            }
            // The Dual mage's spellbook (2026-10-12): what each spell puts in
            // the world on the frame it comes out, besides its own blow. The
            // stars on her, the Shadowstep and the Phase that move her, the
            // level jump, the sigils written by Smite, Sanctuary and the
            // Descent's landing, and Judgement's outer columns. See
            // `docs/design/exploration/0015_dual_mage_spellbook.md`.
            if p.class == Class::DualMage {
                match kind {
                    moves::dual::BINARY => self.raise_the_stars(i),
                    moves::dual::SHADOWSTEP => self.shadowstep(i),
                    moves::dual::PHASE => self.phase(i, true),
                    // **The Equinox blinks as she leaves the floor** (2026-10-12):
                    // Phase's blink along the crosshair, rising out of it and
                    // carrying on, and on top of that the jump as high as her
                    // lower bar is full. The middle click's jump attack is the
                    // mobility one, and it is the balanced one, so it can be
                    // thrown through a jump section without burning her.
                    moves::dual::EQUINOX => {
                        self.phase(i, false);
                        let lift = t::equinox_lift().mul(dual::balance(&p));
                        self.players[i].vel.y = self.players[i].vel.y.add(lift);
                        self.players[i].grounded = false;
                    }
                    moves::dual::CHAIN_LIGHTNING => self.chain_lightning(i),
                    moves::dual::SMITE => {
                        self.write_sigil(i, p.aim_at(), t::sigil_radius(), p.thrown_at)
                    }
                    moves::dual::SANCTUARY | moves::dual::DESCENT => {
                        let floor = self.terrain().ground_under(p.pos);
                        let at = V3::new(p.pos.x, floor.min(p.pos.y), p.pos.z);
                        let radius = if kind == moves::dual::SANCTUARY {
                            t::sanctuary_radius()
                        } else {
                            t::sigil_radius()
                        };
                        self.write_sigil(i, at, radius, p.thrown_at);
                    }
                    moves::dual::JUDGEMENT => self.bring_down_the_columns(i),
                    _ => {}
                }
            }
            // The Air ball let go: it rolls from where it was raised, the way
            // the release's aim says. See `roll_the_ball`.
            if p.class == Class::Elementalist && kind == moves::elementalist::AIR_BALL {
                self.send_the_ball(i);
            }
            // The Fire carpet, laid out along her look from a little ahead of
            // her. Along the line `crate::aim` solved -- the crosshair's, not
            // the chest's -- so it lies where she is looking.
            if p.class == Class::Elementalist && kind == moves::elementalist::FIRE_CARPET {
                let dir = p.aim_path.dir();
                let near = p.aim_path.from.add(dir.scale(t::carpet_ahead()));
                spawn_the_only_one(
                    &mut self.effects,
                    Effect::cast(
                        EffectKind::FireCarpet,
                        i as u8,
                        p.class,
                        kind,
                        near,
                        dir,
                        m.reach,
                    ),
                );
            }
            // The Fire fountain: the burst and the wash, at the feet she is
            // leaving. Where she stood, not where she is going.
            if p.class == Class::Elementalist && kind == moves::elementalist::FIRE_FOUNTAIN {
                spawn_the_only_one(
                    &mut self.effects,
                    Effect::cast(
                        EffectKind::Fountain,
                        i as u8,
                        p.class,
                        kind,
                        p.pos,
                        V3::ZERO,
                        t::fountain_radius(),
                    ),
                );
            }
            if p.class == Class::Elementalist && kind == moves::elementalist::DOWNDRAFT {
                self.raise_the_draft(i, EffectKind::Downdraft);
                let drive = Fx::ZERO.sub(t::downdraft_drive());
                self.players[i].vel.y = self.players[i].vel.y.min(drive);
                self.players[i].grounded = false;
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
                } else if p.class == Class::Elementalist && kind == moves::elementalist::TREMOR {
                    // On her own feet: Tremor is Quake with its centre set
                    // to where she is standing, so the stone it leaves comes
                    // up under her.
                    (p.pos, V3::ZERO)
                } else {
                    (p.aim_at(), V3::ZERO)
                };
                // **Fire on a stone lights it.** A pillar -- tapped or held --
                // cast on a stone's top sets fire to the stone. See
                // `class::Structure::lit`.
                if leaves == EffectKind::FirePillar {
                    stones::light_within(&mut self.players, from, m.radius);
                }
                // A channelled move goes as far as it was wound to, not as far
                // as its row says it could -- see `step_channel`. Only the
                // charge that buys reach: the Strike's `channelled` is a share
                // of a hold, not a distance.
                let mut born = Effect::cast(leaves, i as u8, p.class, kind, from, along, m.reach);
                born.power = depth(&p);
                // The Blood mage's cloud is as wide as its row's radius -- its
                // reach is how far away it can be put.
                if p.class == Class::BloodMage && leaves == EffectKind::Plague {
                    born.reach = m.radius;
                }
                // **The Strike spends the pillar.** What the hold concentrated
                // into the hit is taken off the burn: a full hold leaves
                // nothing standing, a tap leaves the pillar as built, and in
                // between a shorter one. See `strike_bonus`.
                if moves::charge(p.class, kind) == Some(moves::Charge::Strike) {
                    born.life = Fx::from_int(born.life as i32)
                        .mul(Fx::ONE.sub(p.channelled))
                        .to_int()
                        .max(0) as u16;
                    // **Less than a tick left is a flash** (2026-10-21, from
                    // play): it used to leave nothing at all, so a full Strike
                    // had no picture and read as a cast that never came out.
                    // The pillar shoots up to its full size over these few
                    // frames (`Effect::progress` is its age over its life) and
                    // is gone, shorter than a tick so it never burns: the
                    // Strike's hit was the whole burn already.
                    let tick = t::effect_tick_frames().max(1);
                    if born.life < tick {
                        born.life = born.life.max(t::strike_flash()).min(tick - 1);
                    }
                }
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
                // **The Black spike's higher tiers** (2026-10-12): three
                // spikes in a triangle round the point, or a field there that
                // comes up in waves. Each is an area that does its own hitting,
                // so the move's disc lands nothing; a stain under any of them
                // erupts as well.
                if leaves == EffectKind::BlackSpike
                    && p.class == Class::BloodMage
                    && kind == moves::blood::BLACK_SPIKE
                {
                    self.players[i].hit_used = true;
                    self.raise_the_tier(i, born);
                    place = false;
                } else if leaves == EffectKind::BlackSpike {
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
                // A quake is the loudest thing a fighter does (P5).
                if leaves == EffectKind::Quake {
                    crate::noise::make(
                        &mut self.lore,
                        crate::noise::NoiseKind::Quake,
                        born.pos,
                        i as u8,
                        0,
                        self.frame,
                    );
                }
            }
        }

        // **What a Blood mage's cast cost lands where the cast does**, on its
        // first active frame (2026-10-13): at the floor under where it was
        // aimed, or at her feet for a move about her own body. After
        // everything the cast put in the world, so a spike's own cost is not
        // the stain it erupts.
        for i in 0..MAX_PLAYERS {
            let p = self.players[i];
            if p.paid == 0 || !p.class.wounds_go_grey() {
                continue;
            }
            let Action::Active { kind, left } = p.action else {
                continue;
            };
            let m = moves::get(p.class, kind);
            if left != m.active {
                continue;
            }
            // **The Black spike leaves no stain** (2026-10-19), its cost
            // included: what she paid for it is only grey.
            if p.class == Class::BloodMage && kind == moves::blood::BLACK_SPIKE {
                self.players[i].paid = 0;
                continue;
            }
            let at = match m.aim() {
                aim::Kind::Grounded | aim::Kind::Skillshot => p.aim_at(),
                // A swing's at the blade's end (2026-10-18): at her feet she
                // would walk it straight back up, and the scythe would cost
                // nothing.
                aim::Kind::Swing if m.strikes() => {
                    let ahead = V3::new(p.facing.x, Fx::ZERO, p.facing.z).normalized();
                    p.pos.add(ahead.scale(live_reach(&p, &m)))
                }
                _ => p.pos,
            };
            let lift = V3::new(at.x, at.y.add(Fx::ONE), at.z);
            let floor = V3::new(at.x, self.terrain().ground_under(lift), at.z);
            self.players[i].paid = 0;
            self.pool_blood(i as u8, p.class, kind, floor, p.paid as i32);
        }

        // The Elementalist's auto and her heavy: two different beams, the same
        // instant resolution, and never both at once for one fighter -- reset
        // once, here, rather than in each, so neither can clobber the other's
        // answer on the frame it is the one actually firing.
        for (i, &held) in frozen.iter().enumerate() {
            if held {
                continue;
            }
            self.players[i].beam_reach = Fx::ZERO;
            self.fire_the_beam(i);
            self.fire_the_cataclysm(i);
        }

        // Hit resolution after both have stepped, so neither ordering wins.
        // In a hunt the fighters cannot hurt each other: the condition is the
        // creature's presence rather than a separate flag, so there is nothing
        // for the two to get out of step about.
        let snapshot = self.players;
        // **The Clash, between fighters** (exploration 0015, C7): two live
        // blows meeting on one frame, at least one of them a Champion's heavy
        // one. The heavier wins outright and the lighter is cancelled into a
        // short stagger; two of a weight bounce both apart.
        let cancelled = if self.pvp() {
            self.clash(&snapshot)
        } else {
            [false; MAX_PLAYERS]
        };
        // Every fighter against every other one: with four, a blow that
        // catches two bodies on one frame lands on both.
        let present = self.present();
        for attacker in 0..present {
            if !self.pvp() {
                break;
            }
            if cancelled[attacker] || snapshot[attacker].health <= 0 {
                continue;
            }
            for defender in 0..present {
                if !self.foes(attacker, defender) || snapshot[defender].health <= 0 {
                    continue;
                }
                if let Some(mut hit) =
                    resolve_hit(&snapshot[attacker], &snapshot[defender], attacker as u8)
                {
                    // The Reaver's cash-in: any blow of hers spends the marks on
                    // whoever it lands on. A no-op for every other blow in the
                    // game, and for anybody carrying no marks.
                    let staggers = cash_the_tally(&mut self.players, attacker, defender, &mut hit);
                    // What the blow is actually worth, before it lands: a killing
                    // hit on someone with forty health left is worth forty, not its
                    // listed damage, so leeching cannot pay out of an empty bar.
                    let dealt = if hit.blocked || hit.parried {
                        0
                    } else {
                        hit.damage.min(snapshot[defender].health)
                    };
                    apply_hit(&mut self.players[defender], hit);
                    // A full shield's Slam staggers what its shake catches
                    // unguarded: the area stagger weight buys. Blocked, it is an
                    // ordinary blocked Slam -- its frames do not change with weight.
                    if !hit.blocked && !hit.parried && bulwark::slam_staggers(&snapshot[attacker]) {
                        self.players[defender].action = Action::Stagger {
                            left: t::slam_full_stagger(),
                        };
                        self.players[defender].stun_total = t::slam_full_stagger();
                    }
                    if staggers {
                        stagger_the_cashed(&mut self.players[defender]);
                        // A body is cut at its middle, and is a body thick.
                        let victim = snapshot[defender].pos;
                        let floor = floor_under(self.arena(), victim);
                        reap(
                            &mut self.effects,
                            &snapshot[attacker],
                            attacker,
                            aim::origin(victim),
                            floor,
                            t::body_radius(),
                        );
                    }
                    let owed = snapshot[attacker]
                        .action
                        .attack_kind()
                        .map(|kind| swing_leech(&snapshot[attacker], kind, dealt))
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
                            // Through a stain, the blow is harder for it, and the
                            // blood it spills is the whole of what it dealt.
                            let mut spilt = dealt;
                            if let Some(hb) = hitbox(&snapshot[attacker]) {
                                let bonus = self.stain_bonus(
                                    attacker,
                                    &m,
                                    hb.from,
                                    hb.to,
                                    Some(defender),
                                    0,
                                );
                                spilt += self.players[defender].wound(bonus);
                            }
                            self.spill_under(
                                attacker as u8,
                                snapshot[attacker].class,
                                kind,
                                &snapshot[defender],
                                spilt,
                            );
                        }
                    }
                    self.players[attacker].hit_used = true;
                    // Landing anything while ascended pulls some health back. A
                    // no-op for everybody else and for her between ascensions.
                    if dealt > 0 {
                        dual::landed_a_hit(&mut self.players[attacker]);
                    }
                    // The aerial spear pays its shove out on contact rather than
                    // on the throw: catch somebody with the fan and it kicks you
                    // the way you are holding, so it is a repositioning tool you
                    // have to earn. Read from the live input rather than from a
                    // direction locked at the throw, because the whole point is
                    // that you choose where to go *as* it connects.
                    champion_fan_boost(&mut self.players[attacker], inputs[attacker]);
                    // The Bulwark's Rebound off a body, blocked or not: it is solid.
                    rebound_off_a_body(&mut self.players[attacker]);
                    // The Dual mage's curse and purge: what her blow lays on them,
                    // or sets off. Only a blow that landed. A no-op for everybody
                    // else.
                    if let Some(kind) = snapshot[attacker].action.attack_kind() {
                        if dealt > 0 && snapshot[attacker].class == Class::DualMage {
                            spell_lands_on_fighter(&mut self.players, attacker, defender, kind);
                        }
                    }
                    // And the other half of the hammer finisher: the knock-up went
                    // out in `resolve_hit`, and this is the Champion following it
                    // off the floor on the same frame.
                    champion_leap_with_them(
                        &mut self.players[attacker],
                        hit.blocked || hit.parried,
                    );
                    // A chain link that actually landed may cut its recovery short
                    // for the next one. Blocked and parried links pay in full, so
                    // the string ends on a defender who answered it -- see
                    // `chain_cancel`.
                    confirm_chain(&mut self.players[attacker], hit.blocked || hit.parried);
                    if !hit.blocked && !hit.parried {
                        let kind = snapshot[attacker].action.attack_kind();
                        feed_the_momentum(&mut self.players[attacker], kind);
                        // A finisher that spent momentum staggers them by what
                        // it spent.
                        if let (Some(k), Mechanic::Forms { fury, .. }) =
                            (kind, snapshot[attacker].mechanic)
                        {
                            if moves::champion::is_full(k) {
                                stagger_by_the_spend(
                                    &mut self.players[defender],
                                    hit.hitstun,
                                    fury.spent,
                                );
                            }
                        }
                    }
                    if hit.parried {
                        self.players[attacker].action = Action::Stagger {
                            left: t::parry_stagger(),
                        };
                        self.players[attacker].stun_total = t::parry_stagger();
                        self.players[defender].parried = PARRY_FLOURISH;
                    }
                    // Both bodies stop, together. A parry is the cleanest contact
                    // in the game and freezes as long as a hit; a block is a blow
                    // that arrived and was taken, and freezes for less.
                    if let Some(kind) = snapshot[attacker].action.attack_kind() {
                        let m = moves::get(snapshot[attacker].class, kind);
                        let hold = impact_freeze(m.hitstop, hit.blocked && !hit.parried)
                            + true_link_freeze(&snapshot[attacker]);
                        freeze(&mut self.players[attacker], hold);
                        freeze(&mut self.players[defender], hold);
                    }
                }
            }
        }

        // The scythe collects: any pool of hers its volume passes over on an
        // active frame is drunk, hit or no hit. The blade is the thing she
        // puts through the blood, and it does not need a body in the way.
        for i in 0..MAX_PLAYERS {
            self.collect_with_the_scythe(i);
        }
        // And so do her feet: a stain she walks into is drunk.
        self.drink_underfoot();

        // A Rush that runs into a body hits it with the shoulder, and that is
        // the string's first hit. See `World::shoulder`.
        for i in 0..MAX_PLAYERS {
            self.shoulder(i);
        }

        // A thrown shield is its own threat while it travels.
        let shields: [Option<Shield>; MAX_PLAYERS] =
            std::array::from_fn(|i| self.players[i].shield());
        for (owner, shield) in shields.iter().enumerate().take(present) {
            let Some(Shield::Flying {
                pos,
                vel,
                outbound,
                travelled,
                weight,
                struck,
                met,
            }) = *shield
            else {
                continue;
            };
            // Once a flight, and never a partner: in a hunt the fighters
            // cannot hurt each other, the same rule every other blow follows.
            if struck || !self.pvp() {
                continue;
            }
            // The first body it reaches, in seat order: once a flight, so a
            // second body on the same frame is not struck.
            let hit_range = t::shield_radius().add(t::body_radius());
            let reached = (0..present).find(|&i| {
                let victim = &self.players[i];
                let d = victim.pos.sub(pos);
                i != owner
                    && victim.health > 0
                    && d.y.abs().raw() < t::body_height().raw()
                    && d.flat_len().raw() < hit_range.raw()
                    && !victim.untouchable()
            });
            if let Some(target) = reached {
                let victim = self.players[target];
                let d = victim.pos.sub(pos);
                {
                    let dir = V3::new(d.x, Fx::ZERO, d.z).normalized();
                    let blocked = victim.action.guarding();
                    // The weight is the *throw's*. On the way home it is the
                    // ordinary recall, cutting what it passes through.
                    let damage = if outbound {
                        bulwark::throw_damage(weight)
                    } else {
                        t::shield_damage()
                    };
                    apply_hit(
                        &mut self.players[target],
                        Hit {
                            damage,
                            hitstun: t::shield_hitstun(),
                            blockstun: t::shield_blockstun(),
                            knockback: t::shield_knockback(),
                            launch: Fx::ZERO,
                            grabs: 0,
                            by: owner as u8,
                            dir,
                            blocked,
                            parried: false,
                            interrupts: true,
                        },
                    );
                    // A loaded one knocks them down: on the floor, whatever
                    // it did to their frames. Blocked, it is a blocked hit.
                    if outbound && !blocked && bulwark::knocks_down(weight) {
                        self.players[target].action = Action::Stagger {
                            left: t::knockdown_frames(),
                        };
                        self.players[target].stun_total = t::knockdown_frames();
                    }
                    // Thrown, contact drops it where it struck -- **and spends
                    // it.** What it carried went into them, so what plants
                    // there is an empty shield rather than a wall sized by a
                    // blow it has already delivered.
                    //
                    // Recalled, it goes on home through them. It used to plant
                    // here too, which did not matter while a planted shield sat
                    // at hand height and a returning one passed over heads; on
                    // the floor, a recall through anybody never came back.
                    self.players[owner].mechanic = Mechanic::Shield(if outbound {
                        Shield::Planted {
                            pos: V3::new(pos.x, self.terrain().ground_under(pos), pos.z),
                            weight: Fx::ZERO,
                        }
                    } else {
                        Shield::Flying {
                            pos,
                            vel,
                            outbound,
                            travelled,
                            weight,
                            struck: true,
                            met,
                        }
                    });
                }
            }
        }

        if self.hunting() {
            self.shields_strike_the_quarry();
            self.trade_with_the_creature();
            self.trade_with_the_pack();
        }
        self.settle_the_clashes();

        self.step_shadows();
        self.burst_the_collected();
        self.step_effects();
        self.fire_lights_the_stones();
        let standing = self.effects;
        let versus = self.pvp();
        let allies: [u8; MAX_PLAYERS] = std::array::from_fn(|i| self.allies(i));
        bolt::step(
            &mut self.bolts,
            &mut self.players,
            &standing,
            versus,
            allies,
            pack::Prey {
                herd: &mut self.monsters,
                critters: &mut self.critters,
                pack: &mut self.pack,
            },
            here,
        );
        debris::step(
            &mut self.debris,
            &mut self.players,
            &standing,
            versus,
            allies,
            pack::Prey {
                herd: &mut self.monsters,
                critters: &mut self.critters,
                pack: &mut self.pack,
            },
            here,
        );
        let mut bursts: gust::Bursts = [None; gust::MAX_GUSTS];
        gust::step(
            &mut self.gusts,
            &mut self.players,
            &standing,
            versus,
            allies,
            pack::Prey {
                herd: &mut self.monsters,
                critters: &mut self.critters,
                pack: &mut self.pack,
            },
            here,
            &mut bursts,
        );
        // What the shots left where they landed: a cloud of embers per
        // burst. **On the floor it is a patch**: a burst that would hang below
        // the ground is stood on it instead, so the same ball reads as a low
        // burning patch rather than a half-buried one, and its top still
        // reaches a standing body. See `effects::EffectKind::Embers`.
        for burst in bursts.into_iter().flatten() {
            let floor = self.terrain().ground_under(burst.at);
            let mut at = burst.at;
            if at.y.sub(burst.radius).raw() < floor.raw() {
                at.y = floor;
            }
            let class = self.players[burst.owner as usize].class;
            spawn_effect(
                &mut self.effects,
                Effect::cast(
                    EffectKind::Embers,
                    burst.owner,
                    class,
                    burst.slot,
                    at,
                    V3::ZERO,
                    burst.radius,
                ),
            );
            // Fire beside a stone lights it; a stone that burst throws its
            // pieces along the shove that burst it.
            stones::light_within(&mut self.players, at, burst.radius);
            if let Some(dir) = burst.debris {
                debris::blast(&mut self.debris, burst.owner, burst.at, dir);
            }
        }
        stones::touch(&mut self.players);
        let present = self.present();
        separate_bodies(&mut self.players, present);
        drag_the_held(&mut self.players);

        // The hunt's lore, a frame on: fire on the floor, hazards aging, a
        // cart rolling.
        self.step_the_lore();

        // The pack's referee, with everything where it ends the frame: which
        // windups began off whose screen, and who has stood under a marker
        // how long (`pack::watch`).
        if let Some(mut brain) = self.pack {
            let field = stones::gather(&self.players);
            let ground = self.terrain();
            let signs = self.signs();
            let scene = Scene {
                stones: &field,
                players: &self.players,
                effects: &self.effects,
                quarry: &self.monsters,
                critters: &self.critters,
                arena: &ground,
            };
            pack::watch(&mut brain, &self.critters, &inputs, &scene, &signs);
            self.pack = Some(brain);
        }

        // A jump course: a fall stands you back on the last gate you reached,
        // and reaching a gate is the run's. After every body has moved, so a
        // fall is read against where the frame left them.
        self.step_course();

        // The valley: a death in a peaceful place is a cairn, a cairn rests
        // you, and a seam everybody is in -- or somebody has held -- takes
        // you both on. A trip replaces the world, so it is the frame's last
        // word.
        if let Some(next) = self.step_valley() {
            *self = next;
            return;
        }

        // **Versus stages** (`crate::versus`): off a floating stage's edge is
        // out, and a hill's point, clocks and respawns decide the round
        // rather than the last one standing.
        if matches!(self.phase, Phase::Fighting) && self.pvp() && !self.valley.on {
            if let Some(pit) = crate::versus::of(self.arena).and_then(|s| s.pit()) {
                for p in self.players.iter_mut() {
                    if p.health > 0 && p.pos.y.raw() < pit.raw() {
                        p.health = 0;
                    }
                }
            }
            if self.hill.on {
                self.step_the_hill();
                return;
            }
        }

        // Knockout check last, so the killing blow is fully applied first.
        if matches!(self.phase, Phase::Fighting) && self.hunting() {
            let standing = self.players.iter().any(|p| p.health > 0);
            // The hunt is won when every creature in it is down, and every
            // small body dead or gone -- or when a defended thing that wins it
            // arrives. It is lost when the hunters are down, or when a defended
            // thing that loses it breaks (P7).
            let winner = if objective::lost(&self.lore) {
                QUARRY
            } else if objective::won(&self.lore)
                || !self.monsters.iter().flatten().any(Monster::alive)
                    && pack::beaten(&self.critters)
            {
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
                self.drop_trophies();
                if self.valley.on {
                    for species in self.hunted().into_iter().flatten() {
                        self.valley.credit(species);
                    }
                }
            }
            self.phase = Phase::RoundOver {
                winner,
                left: t::round_over_frames(),
            };
            return;
        }
        if matches!(self.phase, Phase::Fighting) && !self.peaceful() {
            // **Last one standing.** Two fighters is the duel it always was;
            // with three or four it is everybody against everybody, and the
            // round ends when one is left -- or none, a double knockout.
            // A seat whose player has left is not in it: with only one
            // player still connected there is nobody to win against.
            let present = self.present();
            let playing = (0..present).filter(|&i| !self.away.has(i)).count();
            let mut standing = self.players[..present]
                .iter()
                .enumerate()
                .filter(|(i, p)| p.health > 0 && !self.away.has(*i));
            let first = standing.next().map(|(i, _)| i);
            if playing >= 2 && standing.next().is_none() {
                let winner = first.map_or(u8::MAX, |i| i as u8);
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
    /// **Every body, where it stands on the map, and its own hash**: what a
    /// region's checksum is made of, and what the watchdog names when two
    /// worlds disagree. The hash is the same bytes `digest` writes for the
    /// body, on a hasher of its own. Positions are on the valley's map
    /// (`World::map_origin` added), so a region drawn on the map holds still
    /// when the frame moves.
    pub fn each_body(&self, mut visit: impl FnMut(Body, V3, u64)) {
        fn own(write: impl FnOnce(&mut Fnv)) -> u64 {
            let mut h = Fnv::new();
            write(&mut h);
            h.finish()
        }
        let origin = self.map_origin();
        for (i, b) in self.bolts.iter().enumerate() {
            if let Some(b) = b {
                visit(
                    Body::Bolt(i as u8),
                    b.pos.add(origin),
                    own(|h| hash_bolt(h, b)),
                );
            }
        }
        for (i, d) in self.debris.iter().enumerate() {
            if let Some(d) = d {
                visit(
                    Body::Shard(i as u8),
                    d.pos.add(origin),
                    own(|h| hash_shard(h, d)),
                );
            }
        }
        for (i, g) in self.gusts.iter().enumerate() {
            if let Some(g) = g {
                visit(
                    Body::Gust(i as u8),
                    g.pos.add(origin),
                    own(|h| hash_gust(h, g)),
                );
            }
        }
        for (i, e) in self.effects.iter().enumerate() {
            if let Some(e) = e {
                visit(
                    Body::Effect(i as u8),
                    e.pos.add(origin),
                    own(|h| hash_effect(h, e)),
                );
            }
        }
        for (i, p) in self.players.iter().enumerate() {
            visit(
                Body::Fighter(i as u8),
                p.pos.add(origin),
                own(|h| hash_player(h, p)),
            );
        }
        for (slot, beast) in self.monsters.iter().enumerate() {
            if let Some(m) = beast {
                let hash = own(|h| hash_monster(h, slot, m));
                visit(Body::Creature(slot as u8), m.pos.add(origin), hash);
            }
        }
        for (i, c) in self.critters.iter().enumerate() {
            if c.present() {
                visit(
                    Body::Critter(i as u8),
                    c.pos.add(origin),
                    own(|h| hash_critter(h, c)),
                );
            }
        }
    }

    /// One region's checksum: the frame, the region, and every body standing
    /// in its checksum zone, each by its slot and its own hash. The fight's
    /// home region also carries the state that belongs to no body -- the
    /// phase, the arena, the pack, the lore, the journey, who has left -- because
    /// a one-arena game has some, and that is where it is counted until it is
    /// made local. See `docs/design/regions.md` §"A region checksum".
    pub fn region_checksum(&self, grid: &Grid, region: RegionId) -> u64 {
        let mut h = Fnv::new();
        h.write_u32(self.frame);
        h.write_u32(region.word());
        self.each_body(|body, at, own| {
            if grid.in_zone(region, at) {
                h.write_u32(body.word());
                h.write_u64(own);
            }
        });
        if region == grid.fight_home(self.map_origin()) {
            hash_phase(&mut h, self.phase);
            h.write_u32(self.arena.0 as u32);
            h.write_u32(self.paused as u32);
            if let Some(p) = &self.pack {
                hash_pack(&mut h, p);
            }
            h.write_u32(self.critters.species.0 as u32);
            hash_lore(&mut h, &self.lore);
            if self.valley.on {
                let v = &self.valley;
                h.write_u32(v.beaten);
                h.write_u32(v.came as u32);
                for c in &v.cairn {
                    h.write_u32(*c as u32);
                }
            }
            for &w in self.away.words() {
                h.write_u32(w);
            }
            if crate::course::of(self.arena).is_some() {
                for r in &self.course {
                    h.write_u32(r.reached as u32 | (r.falls as u32) << 8);
                    h.write_u32(r.started);
                    h.write_u32(r.time);
                }
            }
        }
        h.finish()
    }

    pub fn checksum(&self) -> u64 {
        self.digest(crate::oven::hash())
    }

    /// The snapshot hash alone, with the tuning left out.
    ///
    /// What pins a fight bit for bit across a change that moves knobs around
    /// without changing any of their values: the Oven's own hash follows its
    /// storage layout, and the state does not care where a number is kept.
    pub fn state_checksum(&self) -> u64 {
        self.digest(0)
    }

    fn digest(&self, tuning: u64) -> u64 {
        let mut h = Fnv::new();
        h.write_u64(tuning);
        h.write_u32(self.frame);
        // Only while set, so a world that is not paused hashes as it always
        // has and every pinned checksum stays pinned.
        if self.paused {
            h.write_u32(0x7061_7573);
        }
        if self.ready.any() {
            h.write_u32(0x7265_6479);
            for w in self.ready.words() {
                h.write_u32(*w);
            }
        }
        for b in &self.bolts {
            match b {
                Some(b) => hash_bolt(&mut h, b),
                None => h.write_u32(0),
            }
        }
        for d in &self.debris {
            match d {
                Some(d) => hash_shard(&mut h, d),
                None => h.write_u32(0),
            }
        }
        for g in &self.gusts {
            match g {
                Some(g) => hash_gust(&mut h, g),
                None => h.write_u32(0),
            }
        }
        for e in &self.effects {
            match e {
                Some(e) => hash_effect(&mut h, e),
                None => h.write_u32(0),
            }
        }
        // Her ground, only while there is any: every fight without a Blood
        // mage in it hashes as it did before there was ground.
        if self.ground.iter().any(|g| g.live()) {
            h.write_u32(0x6B0D);
            for g in &self.ground {
                h.write_i32(g.at.x.raw());
                h.write_i32(g.at.y.raw());
                h.write_i32(g.at.z.raw());
                h.write_i32(g.radius().raw());
                h.write_u32(g.life as u32 | (g.owner as u32) << 16);
            }
            for word in 0..4 {
                h.write_u32((self.ground_held >> (32 * word)) as u32);
            }
        }
        for p in &self.players {
            hash_player(&mut h, p);
        }
        // A slot at a time. An empty slot writes a zero, except that empty
        // slots after the last creature write nothing -- so a world with one
        // creature, or none, hashes exactly as it did before there were two
        // slots, and the Ridgeback's pinned hunts (`tests/ridgeback_pin.rs`)
        // still mean what they say.
        let last = self.monsters.iter().rposition(Option::is_some).unwrap_or(0);
        for (slot, beast) in self.monsters.iter().enumerate().take(last + 1) {
            let Some(m) = beast else {
                h.write_u32(0);
                continue;
            };
            hash_monster(&mut h, slot, m);
        }
        hash_phase(&mut h, self.phase);
        // The hill, only where there is one: every other fight hashes as it
        // did before there were hills.
        if self.hill.on {
            let k = &self.hill;
            h.write_u32(0x4111);
            h.write_u32(k.locked as u32 | (k.owner as u32) << 16 | (k.capper as u32) << 24);
            h.write_u32(k.progress);
            h.write_u32(k.clock[0] as u32 | (k.clock[1] as u32) << 16);
            for w in k.respawn {
                h.write_u32(w as u32);
            }
        }
        // The arena, after everything else and only when it is not the
        // proving ground: so a fight there hashes exactly as it did before
        // arenas were data, and the pinned hunts still mean what they say.
        if self.arena != ArenaId::PROVING_GROUND {
            h.write_u32(0xA0 | (self.arena.0 as u32) << 8);
        }
        // The valley's journey, only in the valley: every other fight hashes
        // exactly as it did before there was one.
        if self.valley.on {
            let v = &self.valley;
            h.write_u32(0x7A11);
            h.write_u32(v.beaten);
            h.write_u32(v.came as u32);
            // Every seat's cairn, two to a word: it was the first two only,
            // so a desync in where seat two or three stands back up went
            // unseen.
            for pair in v.cairn.chunks(2) {
                h.write_u32(pair[0] as u32 | (*pair.get(1).unwrap_or(&0) as u32) << 16);
            }
            // The ranges' rests and the live one's rousing, only once one has
            // been met: every journey before there were ranges hashes as it
            // did.
            if v.rested.iter().any(|&f| f != 0) || v.roused || v.stock != 0 {
                h.write_u32(0x4A26);
                for &f in &v.rested {
                    h.write_u32(f);
                }
                h.write_u32(v.roused as u32);
                h.write_u32(v.stock as u32);
            }
            // A hunt out of its home (exploration 0023), only once one has
            // left it: every journey before creatures roamed hashes as it
            // did.
            let out = v.den.is_some_and(|d| d != self.arena);
            if out || v.away != 0 || v.homing || v.wake {
                h.write_u32(0x2A3);
                h.write_u32(v.den.map_or(0, |d| d.0 as u32 + 1));
                h.write_u32(v.away as u32 | (v.homing as u32) << 16 | (v.wake as u32) << 17);
            }
            // What is carried, fed and lying about, and where the morphs
            // stand -- only once anything has been: every journey before
            // there were eggs hashes as it did.
            if self.keep_touched() {
                h.write_u32(0xE66);
                self.hash_keep(&mut h);
            }
        }
        // Who has left, only when somebody has: every match with everybody
        // still in it hashes exactly as it did before anybody could leave.
        if self.away.any() {
            h.write_u32(0xA3A7);
            for &w in self.away.words() {
                h.write_u32(w);
            }
        }
        // The crowd field, only the streams that have heard something: a
        // fight next to nobody hashes as it did before there was a field.
        for s in self
            .crowd
            .streams
            .iter()
            .filter(|s| s.on && s.last != crate::crowd::Aggregate::NONE)
        {
            h.write_u32(0xC20D);
            h.write_u32(s.region.word());
            let a = &s.last;
            h.write_u32(a.epoch as u32 | (a.fighters as u32) << 16);
            h.write_u32(a.middle[0] as u16 as u32 | (a.middle[1] as u16 as u32) << 16);
            h.write_u32(u32::from_le_bytes([
                a.drift[0] as u8,
                a.drift[1] as u8,
                a.creatures,
                0,
            ]));
            h.write_u32(a.hurt as u32);
            h.write_u32(a.doors);
            h.write_i32(s.crowd.raw());
            h.write_i32(s.heat.raw());
            h.write_i32(s.flow[0].raw());
            h.write_i32(s.flow[1].raw());
        }
        // A course's runs, only in a course: every other fight hashes exactly
        // as it did before there were any.
        if crate::course::of(self.arena).is_some() {
            for r in &self.course {
                h.write_u32(r.reached as u32 | (r.falls as u32) << 8);
                h.write_u32(r.started);
                h.write_u32(r.time);
            }
        }
        // The pack and its bodies, last, and only when there is one: a fight
        // without small bodies hashes exactly as it did before there were any.
        if let Some(p) = &self.pack {
            hash_pack(&mut h, p);
        }
        if self.pack.is_some() || self.critters.any() {
            h.write_u32(self.critters.species.0 as u32);
            for c in self.critters.iter() {
                hash_critter(&mut h, c);
            }
        }
        // The hunt's lore, last, and only when something is in it: a fight
        // with nothing on the floor, nothing heard and nothing to defend
        // hashes exactly as it did before there was any.
        if !self.lore.is_blank() {
            hash_lore(&mut h, &self.lore);
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
    /// The Dual mage's auto on its last frame: its **sweet spot**, the ball
    /// arrived, and it hits harder. See `tuning::orb_sweet`.
    pub sweet: bool,
}

impl Hitbox {
    /// The middle of the volume. What a single-point overlay wants to draw at,
    /// what the browser build reports, and the same point as `from` for a disc.
    pub fn centre(&self) -> V3 {
        self.from
            .add(self.to.sub(self.from).scale(Fx::ONE.div(Fx::from_int(2))))
    }

    /// The point of a flat volume nearest `at` along the floor: its one
    /// point for a disc, and the nearest point of the strip for one stretched
    /// from her body (the Reave's, see [`hitbox`]). A flat volume is tested
    /// by the floor distance from here; everything that tests one asks this.
    pub fn flat_nearest(&self, at: V3) -> V3 {
        crate::math::flat_segment_nearest(at, self.from, self.to)
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
    // **And the Bulwark's weight changes how wide Slam's shake is.** Here, in
    // the one description of an attack's volume, so the overlay draws the
    // shake at the radius it is tested at. See `crate::bulwark`.
    m.radius = bulwark::slam_radius(p, kind, m.radius);
    // **And a held Strike is the pillar's exact shape.** The column of flame
    // that lands where a held pillar would have stood is as wide as the
    // pillar's grown base, whatever the move's own disc says. See
    // `strike_bonus`.
    if moves::charge(p.class, kind) == Some(moves::Charge::Strike) && p.channelled.raw() > 0 {
        m.radius = m.radius.max(t::pillar_base_radius());
    }
    // **And a Champion's finisher is as big as the meter it spent**, as what
    // it puts in the world is (`World::champion_comes_out`): its own sweep
    // reaches its share of `momentum_spend` further. One for everything else.
    m.reach = m.reach.mul(Fx::ONE.add(spent_share_of(p, kind)));
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
        sweet: false,
    };
    // **The Flurry dash's cascade** (2026-10-13): the blow that is out, as its
    // stretch of the line from where she stood to the disk; nothing once they
    // are all out and the disk stands. See `moves::Shape::Cascade`.
    if m.shape == moves::Shape::Cascade {
        let (blows, blow) = moves::flurry_blow(&m, left);
        let blow = blow?;
        let (a, d) = moves::flurry_line(p.aim_path.from, p.aim_path.to);
        let n = blows.max(1) as i32;
        return Some(Hitbox {
            from: crate::math::lerp3(a, d, Fx::ratio(blow as i32, n)),
            to: crate::math::lerp3(a, d, Fx::ratio(blow as i32 + 1, n)),
            radius: m.radius,
            flat: false,
            hits_crouching: m.hits_crouching,
            unblockable: m.unblockable,
            spent: p.hit_used,
            sweet: false,
        });
    }
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
        // From the disk over her head to the point the crosshair was on: the
        // whole line, every active frame. The disk follows her; the point
        // was locked when the blast began.
        aim::Kind::Overhead => (aim::overhead(p.pos), p.aim_path.to, false),
        // Where the thing was planted. The burst that comes with it has to be
        // there too, or the ability is two abilities pointing different ways.
        aim::Kind::Grounded => return Some(disc(p.aim_at())),
        // The crack's line, from the held stone to wherever the hold bought.
        // Fissure puts out no volume of its own (`moves::shape`), so this arm
        // is never reached today; it says what the volume *would* be so the
        // answer is total rather than a wildcard.
        aim::Kind::Racing => (p.aim_path.from, p.aim_path.to, true),
        // Not aimed at a point -- a body moving. Live rather than locked,
        // because a move that can be thrown on the move has to travel with the
        // body.
        aim::Kind::Swing => match m.shape {
            moves::Shape::None => return None,
            // The original: a disc at arm's length, along the line the
            // swing came out on -- the facing's yaw, the camera's pitch.
            //
            // A flat disc has no top and no bottom, so a stoop to meet
            // something short (`aim::stoop`) has nothing to bring down -- it
            // would only pull the disc in toward her. Pointed at a short body,
            // it goes out level along the facing, at its whole reach.
            moves::Shape::Cylinder if p.stoop.raw() < 0 => {
                return Some(disc(p.pos.add(p.facing.scale(m.reach))));
            }
            // **The Reave's cut reaches back to her** (2026-10-21, from
            // play): its dash is a fixed 2.5 m, so thrown from on top of
            // somebody it ran her onto them and the disc landed past them --
            // a whiff, and only the shadow's copy connected, which marks
            // rather than cashes. 0013 says the cut is plain from there, not
            // gone: so the disc is stretched into a strip from her body,
            // and only its far end is the sweet spot (`reave_sweet`).
            moves::Shape::Cylinder
                if p.class == Class::ShadowReaver && kind == moves::reaver::REAVE =>
            {
                return Some(Hitbox {
                    from: p.pos,
                    ..disc(p.pos.add(p.aim_dir().scale(m.reach)))
                });
            }
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
                // The second Sweep of a string comes back the other way.
                let back = back != (string_place(p, kind) == Some(1));
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
                // Stooped to meet something short (`aim::stoop`), the point
                // still reaches as far across the floor as it would have level:
                // a spear lowered at a gnawer is lowered, not shortened.
                let dir = p.aim_dir();
                let across = dir.flat_len();
                let out = if p.stoop.raw() < 0 && across.raw() > 0 {
                    out.div(across)
                } else {
                    out
                };
                (hub, hub.add(dir.scale(out)), false)
            }
            // **A ball on a curved flight** (2026-10-13): the stretch of
            // `moves::orb_at`'s path the ball covered this frame, so a ball
            // crossing metres a frame cannot step over a body, and on the last
            // frame the sweet spot -- a bubble of its own at the end of the
            // flight, dead in front of her chest at full reach. What it meets
            // on the way it hits, for no bonus: standing a reach away is the
            // decision the sweet spot rewards.
            moves::Shape::Cascade => return None,
            moves::Shape::Orb => {
                let dir = moves::orb_line(p.facing, p.aim_dir(), p.grounded, p.stoop);
                if left == 0 {
                    let end = moves::orb_at(p.pos, p.facing, dir, &m, Fx::ONE);
                    return Some(Hitbox {
                        from: end,
                        to: end,
                        radius: t::orb_sweet_radius(),
                        flat: false,
                        hits_crouching: m.hits_crouching,
                        unblockable: m.unblockable,
                        spent: p.hit_used,
                        sweet: true,
                    });
                }
                let (was, now) = moves::orb_progress(&m, left);
                (
                    moves::orb_at(p.pos, p.facing, dir, &m, was),
                    moves::orb_at(p.pos, p.facing, dir, &m, now),
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
        sweet: false,
    })
}

/// **The Reave's sweet spot** (2026-10-10): what the hit is worth when the
/// body it lands on is within `tuning::reave_sweet` of the middle of the
/// blade's disc -- the end of the dash, and the edge she drew it with. One
/// for every other move. `inside` answers whether the body is in a disc of
/// a given middle and radius, so the fighter and the creature each ask it of
/// their own shape.
fn reave_sweet(
    attacker: &Player,
    kind: u8,
    box_out: &Hitbox,
    inside: impl Fn(V3, Fx) -> bool,
) -> Fx {
    if attacker.class != Class::ShadowReaver || kind != moves::reaver::REAVE {
        return Fx::ONE;
    }
    // The middle of the cut's disc, at the far end of its strip.
    if inside(box_out.to, t::reave_sweet()) {
        t::reave_sweet_damage()
    } else {
        Fx::ONE
    }
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
    // **The vessel bleeds her** while it holds a charge: a stored spell is a
    // threat both players can see, with a clock on it.
    if p.vessel != NO_VESSEL && frame % (t::vessel_bleeds_every() as u32) == 0 {
        p.spend_health(1);
    }
    if p.grey <= 0 {
        return;
    }
    // **It holds, then it goes** (2026-10-13): the grace after a wound is the
    // window to spend it in, and past it the fade is quick.
    if p.grey_rest > 0 {
        p.grey_rest -= 1;
        return;
    }
    let rate = t::grey_fade().max(0);
    let hz = crate::TICK_HZ as i32;
    let k = (frame % crate::TICK_HZ) as i32;
    let faded = rate * (k + 1) / hz - rate * k / hz;
    p.grey = (p.grey - faded).max(0);
}

/// What a held Strike is worth over the pillar's own hit: the share of the
/// pillar's whole burn the hold concentrated, at `tuning::strike_worth`.
///
/// Nothing on a tap (`channelled` is zero), and nothing on any other move or
/// class. The burn it is worth is exactly the burn `advance` takes off the
/// pillar's life, so a full Strike is the pillar's total arriving at once and
/// nothing left to stand in.
fn strike_bonus(p: &Player, kind: u8) -> i32 {
    if moves::charge(p.class, kind) != Some(moves::Charge::Strike) || p.channelled.raw() <= 0 {
        return 0;
    }
    Fx::from_int(t::pillar_burn_total())
        .mul(p.channelled)
        .mul(t::strike_worth())
        .to_int()
        .max(0)
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
    if box_out.spent || defender.untouchable() {
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
    let tipped = box_out.sweet;
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
    // The Reave's sweet spot: the middle of the blade's disc, at the end of
    // its dash. See [`reave_sweet`].
    let sweet = reave_sweet(attacker, kind, &box_out, |c, r| {
        V3::new(defender.pos.x.sub(c.x), Fx::ZERO, defender.pos.z.sub(c.z))
            .flat_len()
            .raw()
            <= r.raw()
    });
    let damage_mul = preying(attacker.class, defender.disabled())
        .mul(sweet)
        .mul(if tipped { t::orb_sweet() } else { Fx::ONE })
        .mul(if on_the_point {
            t::sweep_tip_damage()
        } else {
            Fx::ONE
        })
        .mul(grey_power(attacker, &m))
        .mul(blood_blow(attacker, kind))
        .mul(true_link_mul(attacker))
        .mul(momentum_mul(attacker, kind))
        .mul(power);

    let reach = box_out.radius.add(t::body_radius());
    let hit_at_all = if box_out.flat {
        // The original rule, kept for every move that has not been given a
        // shape: flat distance from the middle of the disc, with no top and no
        // bottom. You cannot duck under one of these or jump over it.
        let delta = defender.pos.sub(box_out.flat_nearest(defender.pos));
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
    // of dragging somebody is that they arrive at *you*, and a blow from
    // the side would otherwise send them backwards past her instead of in. A
    // push is still the facing, because what a shove does is send them the
    // way the blow was travelling.
    //
    // The Dual mage's **sweet spot** multiplies it either way: the light
    // auto's real knockback lives at full reach, and on the dark arm the same
    // frame hauls them that much further in. See `tuning::orb_sweet_shove`.
    let shove = m.knockback.mul(power).mul(if tipped {
        t::orb_sweet_shove()
    } else {
        Fx::ONE
    });
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

    // The Bulwark's Slam spends his weight and his fall. The base for
    // everybody else and every other move.
    let damage = bulwark::slam_damage(attacker, kind, m.damage);
    // And the Elementalist's held Strike adds the burn it took off the
    // pillar, on top of the pillar's own hit.
    let damage = damage + strike_bonus(attacker, kind);
    Some(Hit {
        damage: Fx::from_int(damage).mul(damage_mul).to_int(),
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

/// **What a Dual mage's spell does to a fighter it landed on**, besides its
/// own blow (2026-10-12): a dark spell lays its curse stacks, the Grasping
/// shade roots, and a light spell **purges** whatever curse is there -- every
/// stack at once, at the power the light was thrown with. See
/// `docs/design/exploration/0015_dual_mage_spellbook.md`.
fn spell_lands_on_fighter(players: &mut [Player; MAX_PLAYERS], by: usize, victim: usize, kind: u8) {
    if players[by].class != Class::DualMage || by == victim {
        return;
    }
    let laid = moves::dual::curses_laid(kind);
    if laid > 0 {
        let v = &mut players[victim];
        dual::lay_curse(&mut v.curse, &mut v.curse_clock, by, laid);
    }
    if kind == moves::dual::GRASPING_SHADE {
        players[victim].slow(t::root_for(), t::root_slow());
    }
    if moves::dual::force(kind) == Some(Force::Light) {
        let power = dual::depth_at(&players[by], Some(kind));
        purge_the_fighter(players, by, victim, power);
    }
}

/// **The purge, on a fighter**: every stack of the curse on them goes off at
/// once, and a big enough one staggers. Spent whoever laid it -- light sets off
/// darkness, whosever it was. Gives back what it dealt.
fn purge_the_fighter(
    players: &mut [Player; MAX_PLAYERS],
    by: usize,
    victim: usize,
    power: Fx,
) -> i32 {
    let n = dual::stacks(players[victim].curse);
    if n == 0 {
        return 0;
    }
    players[victim].curse = dual::NO_CURSE;
    players[victim].curse_clock = 0;
    let dealt = players[victim].wound(dual::purge_damage(n, power));
    let q = &mut players[victim];
    if dual::purge_staggers(n) && !q.untouchable() && q.health > 0 {
        let stagger = t::purge_stagger().max(1);
        q.stun_total = q.stun_total.max(stagger);
        q.action = Action::Stagger { left: stagger };
    }
    if dealt > 0 {
        dual::landed_a_hit(&mut players[by]);
    }
    dealt
}

/// **What a Dual mage's spell does to a creature it landed on**: the
/// fighter's rule on a body with no `Player`. A root is a slow on the animal;
/// a purge is damage, and no stagger -- what a creature feels is its own
/// flinch and poise rules' business.
fn spell_lands_on_creature(
    beast: &mut monster::Monster,
    players: &mut [Player; MAX_PLAYERS],
    by: usize,
    kind: u8,
) {
    if players[by].class != Class::DualMage || !beast.alive() {
        return;
    }
    let laid = moves::dual::curses_laid(kind);
    if laid > 0 {
        dual::lay_curse(&mut beast.curse, &mut beast.curse_clock, by, laid);
    }
    if kind == moves::dual::GRASPING_SHADE {
        root_the_creature(beast, t::root_for());
    }
    if moves::dual::force(kind) == Some(Force::Light) {
        let power = dual::depth_at(&players[by], Some(kind));
        purge_the_creature(beast, &mut players[by], power);
    }
}

/// The purge, on a creature.
fn purge_the_creature(beast: &mut monster::Monster, by: &mut Player, power: Fx) -> i32 {
    let n = dual::stacks(beast.curse);
    if n == 0 || !beast.alive() {
        return 0;
    }
    beast.curse = dual::NO_CURSE;
    beast.curse_clock = 0;
    let dealt = dual::purge_damage(n, power).min(beast.health).max(0);
    beast.health -= dealt;
    if dealt > 0 {
        dual::landed_a_hit(by);
    }
    dealt
}

/// A creature held where it stands: a slow to `root, speed while rooted`.
fn root_the_creature(beast: &mut monster::Monster, frames: u16) {
    beast.slowed = beast.slowed.max(frames);
    beast.slow_mul = if beast.slow_mul.raw() > 0 {
        beast.slow_mul.min(t::root_slow())
    } else {
        t::root_slow()
    };
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
    let facing_it = in_guard_arc(defender.pos, defender.facing, from, t::guard_arc_cos());
    let guarding = defender.action.guarding() && facing_it;
    let parried =
        facing_it && matches!(defender.action, Action::Guard { held } if held < t::parry_window());
    (guarding, parried)
}

/// **Is `from` inside a guard's arc**: a defender at `at`, facing `facing`,
/// whose arc is `arc_cos` (the cosine of its half-width). The one cone test
/// both kinds of defender use -- a fighter's guard (`guard_against`) and a
/// creature's (the Mantis, `species::mantis::fight::guard`), which adds an
/// elevation of its own on top.
pub fn in_guard_arc(at: V3, facing: V3, from: V3, arc_cos: Fx) -> bool {
    let toward = from.sub(at).normalized();
    facing.dot(toward).raw() >= arc_cos.raw()
}

/// How long a blow of `hitstop` frames freezes for, blocked or not.
fn impact_freeze(hitstop: u16, blocked: bool) -> u16 {
    if blocked {
        (hitstop as u32 * t::freeze_on_block() as u32 / 100) as u16
    } else {
        hitstop
    }
}

/// **A heavy blow on a creature holds both bodies** (2026-10-21, from play:
/// strong hits on a creature did not land like strong hits). A fighter's
/// blow from a move whose impact freeze is at least
/// `tuning::creature_heavy_from` freezes the fighter and the creature for
/// that length -- the same length, so neither gains a frame -- exactly as a
/// blow between two fighters does. Lighter blows freeze nothing, so a string
/// of quick cuts keeps its pace against something this size.
fn freeze_on_the_creature(p: &mut Player, beast: &mut Monster, attacker: &Player, m: &moves::Move) {
    if m.hitstop < t::creature_heavy_from() {
        return;
    }
    let hold = m.hitstop + true_link_freeze(attacker);
    freeze(p, hold);
    beast.frozen = beast.frozen.max(hold.min(u8::MAX as u16) as u8);
}

/// Hold a body still for `frames`, or for whatever freeze it is already in if
/// that is longer. Two blows landing on one frame are one impact, not two in
/// a row.
fn freeze(p: &mut Player, frames: u16) {
    p.frozen = p.frozen.max(frames.min(u8::MAX as u16) as u8);
}

/// What a fighter's input is this frame, given the freeze.
///
/// Held: every button pressed is remembered and the input goes on as it came
/// in, for whoever reads it -- nothing does, except the one step that is
/// skipped. Out of it: whatever was remembered is pressed on this frame, once.
/// Buttons only; where the stick points is always where it points now.
fn thaw(p: &mut Player, input: Input) -> Input {
    find_the_beat(p, input);
    if p.frozen > 0 {
        p.frozen_bits |= input.bits & Input::PRESSES;
        return input;
    }
    let kept = p.frozen_bits;
    p.frozen_bits = 0;
    input.with(kept)
}

pub(crate) fn apply_hit(defender: &mut Player, hit: Hit) {
    // **The Clash** (exploration 0015, C7): a blow that meets a Champion's
    // heavy one takes nothing from him. Here, where every blow in the game
    // lands on a fighter, so every attack there is -- a species' own, a
    // critter's, a fighter's -- meets it without a line of its own. Who threw
    // it is knocked out of it after (`World::settle_the_clashes`).
    if meets_a_clash(defender, &hit) {
        return;
    }
    // A Champion mid-Rush is braced: most blows cost him a share and do not
    // stop him, and one in the run's last frames costs nothing at all.
    let Some(hit) = brace_the_rush(defender, hit) else {
        return;
    };
    // **A blow one fighter landed on another.** The swell, weight and the
    // string are all about two fighters running along bars of the same size;
    // a creature's blow, a critter's or a hazard's does exactly what its table
    // says. See `docs/design/stun.md`.
    let duel = (hit.by as usize) < MAX_PLAYERS;
    // Taken on the shield, either way: stored as weight. A parry stores more.
    // See `crate::bulwark` -- this is the one place a blocked blow is
    // resolved, which is why the creature's blows load it without asking.
    if hit.blocked || hit.parried {
        bulwark::load(defender, hit.damage, hit.parried);
    }
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
        // A heavy shield moves less. One at empty, `heavy_pushback` at the cap.
        //
        // And a heavy *class* moves less, between fighters: the same weight
        // that divides a hit divides a blocked one. Blocked pushback never
        // swells -- the swell is about damage taken, and a block takes none.
        let knockback = hit.knockback.mul(bulwark::pushback(defender));
        let knockback = if duel {
            knockback.div(t::weight_of(defender.class))
        } else {
            knockback
        };
        defender.vel.x = hit.dir.x.mul(knockback);
        defender.vel.z = hit.dir.z.mul(knockback);
    } else {
        defender.wound(hit.damage);
        if hit.damage > 0 {
            hurt_the_momentum(defender);
        }
        // **How far a fighter's blow throws another**: swelled by what the
        // victim has already lost -- read after this blow's damage, so the
        // killing blow carries the largest swell there is -- and divided by
        // what they weigh. One multiplier for the whole launch, sideways and
        // up alike, so a move keeps the angle it was authored with and only
        // the distance grows. A creature's blow does what its table says.
        //
        // The shove starts above the table at full health and climbs to the
        // same ceiling (`tuning::swell_start`); the launch starts at the
        // table, because the follow-ups into the air are tuned to its height.
        let (carry, lift_carry) = if duel {
            let weight = t::weight_of(defender.class);
            (
                knockback_swell(defender, hit.damage, t::swell_start()).div(weight),
                knockback_swell(defender, hit.damage, Fx::ONE).div(weight),
            )
        } else {
            (Fx::ONE, Fx::ONE)
        };
        let shove = hit.knockback.mul(carry);
        let lift = hit.launch.mul(lift_carry);
        defender.vel.x = hit.dir.x.mul(shove);
        defender.vel.z = hit.dir.z.mul(shove);
        if hit.grabs > 0 {
            defender.seized(hit.by, hit.grabs, 0);
            // A grab that launches cannot do both at once: the hold zeroes
            // their velocity every frame. The launch waits for the release.
            defender.thrown = lift.max(Fx::ZERO);
        } else if hit.interrupts {
            // Writing the action is the interrupt, and it happens even when
            // `hitstun` is zero -- `HitStun { left: 0 }` is one frame of
            // nothing, but the move it replaced is gone. Anything that must
            // land without stopping its victim has to skip this branch rather
            // than tune the number to zero. See `Hit::interrupts`.
            let stun = if duel {
                swelled_hitstun(defender, hit.hitstun)
            } else {
                hit.hitstun
            };
            if duel {
                defender.taken = defender.taken.saturating_add(1);
            }
            defender.stun_total = stun;
            defender.action = Action::HitStun { left: stun };
        }
        // Anything that is not itself a spike clears one that was banked: being
        // caught on the way down and thrown somewhere else is not the landing
        // the spike was charging for.
        defender.slam = Fx::ZERO;
        if hit.grabs > 0 {
            // Banked above; the hold decides where they go until it ends.
        } else if lift.raw() > 0 {
            // Taken off the ground. Launch *sets* vertical speed rather than
            // adding to it, so being hit on the way down does not cancel out.
            defender.vel.y = lift;
            defender.grounded = false;
        } else if lift.raw() < 0 {
            // Spiked. The same rule pointed the other way, and the speed is
            // banked so that the landing can charge for it -- see
            // `Player::slam`. Terminal velocity is what the ground actually
            // sees, so banking the intent rather than the arrival would
            // overstate it.
            defender.vel.y = lift;
            defender.grounded = false;
            defender.slam = lift.abs();
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
        // And the Blood mage's haul, for the same reason: a pull you can be
        // hit out of and keep is a commitment with nothing on the other side of
        // it. The arms let go.
        defender.haul = 0;
    }
}

/// The share of a fighter's own bar already gone: zero untouched, one at
/// death. Smash's percent, upside down.
fn hurt(p: &Player) -> Fx {
    let full = p.full_health().max(1);
    Fx::ratio((full - p.health).clamp(0, full), full)
}

/// What a fighter's blow is multiplied by against this victim, before their
/// weight: `start` at full health, rising in a straight line to the ceiling
/// at death.
///
/// The ceiling has two terms, as Smash has: one for dying at all, the other
/// for the share of their bar this blow took, so a committed move scales
/// harder against a hurt opponent than a poke does. A start above one is a
/// slower climb to the same ceiling, never a higher one.
fn knockback_swell(victim: &Player, damage: i32, start: Fx) -> Fx {
    let full = victim.full_health().max(1);
    let share = Fx::ratio(damage.clamp(0, full), full);
    let ceiling = Fx::ONE
        .add(t::swell_knockback())
        .add(t::swell_blow().mul(share));
    let rise = ceiling.sub(start).max(Fx::ZERO);
    start.add(hurt(victim).mul(rise))
}

/// How much further this fighter would be thrown than an untouched one: the
/// readable half of the swell, for the HUD, where Smash shows its percent. It
/// leaves out the blow's own term, which belongs to whichever move lands next,
/// and folds in the class's weight.
pub fn launch_scale(p: &Player) -> Fx {
    let ceiling = Fx::ONE.add(t::swell_knockback());
    let start = t::swell_start();
    start
        .add(hurt(p).mul(ceiling.sub(start).max(Fx::ZERO)))
        .div(t::weight_of(p.class))
}

/// A fighter's blow's hitstun, swelled by what the victim has lost.
///
/// Grows far more slowly than the knockback does, and the gap between the two
/// is the combo design: stun grows and moves start to link, knockback grows
/// faster and throws the victim out of reach again. **And it fades through a
/// string**: each hit taken without a turn back keeps `combo_decay` of the
/// last one's swell, so a ranged move that re-aims wherever they landed runs
/// out of the extra frames too. Never below the move table's own number.
fn swelled_hitstun(victim: &Player, base: u16) -> u16 {
    let mut kept = t::swell_hitstun();
    for _ in 0..victim.taken.min(16) {
        kept = kept.mul(t::combo_decay());
    }
    let swell = Fx::ONE.add(hurt(victim).mul(kept));
    Fx::from_int(base as i32)
        .mul(swell)
        .to_int()
        .clamp(base as i32, u16::MAX as i32) as u16
}

impl World {
    /// **What a full charge does as it comes out** (exploration 0015, C5),
    /// besides its own volume: the crescent leaves the blade, the ground
    /// breaks round the hammer, the dashes start, the cuts are left hanging
    /// along the line he will run, and the javelin is thrown with him after
    /// it. On the first active frame, like everything else a move does.
    fn champion_comes_out(&mut self, who: usize, kind: u8) {
        use moves::champion as c;
        let p = self.players[who];
        let flat = |v: V3| V3::new(v.x, Fx::ZERO, v.z).normalized();
        // **What the finisher spent** (exploration 0015, the fourth round):
        // what it puts in the world is bigger and hits harder by its share,
        // and carries the share in `banked` for the stagger it buys.
        let spent = match p.mechanic {
            Mechanic::Forms { fury, .. } => fury.spent,
            _ => 0,
        };
        let power = Fx::ONE.add(spend_share(spent));
        let cast = |kind_of: EffectKind, from: V3, dir: V3, reach: Fx| {
            let mut born = Effect::cast(kind_of, who as u8, p.class, kind, from, dir, reach);
            born.power = power;
            born.banked = spent as i32;
            born
        };
        match kind {
            c::CRESCENT => {
                let born = cast(
                    EffectKind::Crescent,
                    p.aim_path.from,
                    p.aim_path.dir(),
                    t::crescent_reach(),
                );
                spawn_effect(&mut self.effects, born);
            }
            c::SHOCKWAVE | c::METEOR => {
                let reach = if kind == c::METEOR {
                    t::meteor_reach()
                } else {
                    t::shockwave_reach()
                }
                .mul(power);
                let mut born = cast(EffectKind::Shockwave, p.pos, V3::ZERO, reach);
                let speed = t::shockwave_speed().max(Fx::ONE);
                // Out to its reach, and left lying there a while: the broken
                // ground stays after the ring has run.
                born.life = (reach.div(speed).div(DT).to_int().max(1) as u16)
                    .saturating_add(t::shockwave_crush());
                spawn_effect(&mut self.effects, born);
            }
            c::THOUSAND_CUTS => {
                let ahead = flat(p.facing);
                let reach = t::cuts_reach();
                let chest = p.pos.add(V3::new(
                    Fx::ZERO,
                    t::body_height().mul(Fx::ratio(1, 2)),
                    Fx::ZERO,
                ));
                spawn_effect(
                    &mut self.effects,
                    cast(EffectKind::Cuts, chest, ahead, reach),
                );
                drive_the_champion(&mut self.players[who], ahead, reach, t::cuts_speed());
            }
            c::HEAVEN_PIERCER => {
                let dir = p.aim_path.dir();
                let reach = p.aim_path.length().max(t::body_radius());
                let mut born = cast(EffectKind::Javelin, p.aim_path.from, dir, reach);
                let speed = t::javelin_speed().max(Fx::ONE);
                born.life = (reach.div(speed).div(DT).to_int().max(1) as u16)
                    .saturating_add(t::javelin_rest());
                spawn_effect(&mut self.effects, born);
                // **He goes after it at once** -- the throw's second half, not
                // a choice later. One weapon that changes shape: it is never
                // out of his hands for longer than it takes him to get there.
                drive_the_champion(&mut self.players[who], flat(dir), reach, t::follow_speed());
            }
            _ => {}
        }
    }

    /// **Whoever threw a blow the Champion clashed is knocked out of it**
    /// (exploration 0015, C7): a creature mid-swing that reaches him flinches;
    /// a critter mid-attack that reaches him is knocked out of it; a fighter
    /// within `tuning::clash_reach` is staggered. Read off who is attacking
    /// and where, so no attack needs a line of its own. A blow from further
    /// off -- a bolt -- is simply spent.
    fn settle_the_clashes(&mut self) {
        for i in 0..MAX_PLAYERS {
            let p = self.players[i];
            let Mechanic::Forms { fury, .. } = p.mechanic else {
                continue;
            };
            let by = fury.clash_by;
            if by == NOBODY {
                continue;
            }
            if let Mechanic::Forms { fury, .. } = &mut self.players[i].mechanic {
                fury.clash_by = NOBODY;
            }
            if (by as usize) < MAX_PLAYERS && by as usize != i {
                let q = &mut self.players[by as usize];
                let apart = V3::new(q.pos.x.sub(p.pos.x), Fx::ZERO, q.pos.z.sub(p.pos.z));
                if apart.flat_len().raw() <= t::clash_reach().raw() {
                    q.action = Action::Stagger {
                        left: t::clash_stagger(),
                    };
                    q.stun_total = t::clash_stagger();
                    q.hit_used = true;
                    freeze(q, t::clash_freeze());
                }
                continue;
            }
            if by != QUARRY {
                continue;
            }
            for slot in 0..MAX_MONSTERS {
                let Some(beast) = self.monsters[slot].as_mut() else {
                    continue;
                };
                if beast.alive()
                    && matches!(beast.doing, Doing::Active { .. })
                    && beast.reaches(p.pos, p.hurt_height(), t::body_radius())
                {
                    beast.doing = Doing::Flinch {
                        left: t::clash_flinch(),
                    };
                    beast.hit_used = true;
                }
            }
            if self.critters.any() {
                let sp = self.critters.sp();
                for c in self.critters.iter_mut() {
                    if c.alive()
                        && c.state == critter::is::ACTIVE
                        && c.reaches(sp, p.pos, p.hurt_height(), t::body_radius())
                    {
                        c.state = critter::is::FLINCH;
                        c.timer = t::clash_flinch();
                        c.set(critter::flag::HIT_USED, true);
                    }
                }
            }
        }
    }

    /// Settle a Clash between the two fighters, if their blows meet this
    /// frame: who is cancelled. See the call in `advance`.
    fn clash(&mut self, snapshot: &[Player; MAX_PLAYERS]) -> [bool; MAX_PLAYERS] {
        let mut cancelled = [false; MAX_PLAYERS];
        let hits = [
            resolve_hit(&snapshot[0], &snapshot[1], 0),
            resolve_hit(&snapshot[1], &snapshot[0], 1),
        ];
        if hits.iter().any(|h| h.is_none()) || !(clashing(&snapshot[0]) || clashing(&snapshot[1])) {
            return cancelled;
        }
        let weight = |p: &Player| {
            p.action
                .attack_kind()
                .map_or(0, |kind| moves::get(p.class, kind).hitstop)
        };
        let (w0, w1) = (weight(&snapshot[0]), weight(&snapshot[1]));
        let losers: &[usize] = if w0 > w1 {
            &[1]
        } else if w1 > w0 {
            &[0]
        } else {
            &[0, 1]
        };
        for &i in losers {
            cancelled[i] = true;
            let q = &mut self.players[i];
            q.action = Action::Stagger {
                left: t::clash_stagger(),
            };
            q.stun_total = t::clash_stagger();
            q.hit_used = true;
        }
        for i in 0..MAX_PLAYERS {
            freeze(&mut self.players[i], t::clash_freeze());
            if clashing(&snapshot[i]) && !cancelled[i] {
                self.players[i].parried = PARRY_FLOURISH;
            }
        }
        cancelled
    }

    /// A full charge's effect landed `dealt` on somebody: its thrower's
    /// momentum is fed as if his own blow had.
    fn feed_the_thrower(&mut self, effect: &Effect, dealt: i32) {
        let owner = effect.owner as usize;
        if dealt > 0 && owner < MAX_PLAYERS && effect.class == Class::Champion {
            feed_the_momentum(&mut self.players[owner], Some(effect.slot));
        }
    }

    /// **The Heaven-piercer torn out** (exploration 0015, C5): a javelin at
    /// rest, and its thrower arrived at it. Everybody round it is hit and
    /// thrown up, and the spear is back in his hands -- it was only ever one
    /// weapon. Nothing happens until he gets there; a javelin he never reaches
    /// fades where it stopped.
    fn tear_out(&mut self, effect: &mut Effect, at: V3) {
        let owner = effect.owner as usize;
        if owner >= MAX_PLAYERS || effect.already_hit(1, owner) {
            return;
        }
        let p = self.players[owner];
        let near = V3::new(p.pos.x.sub(at.x), Fx::ZERO, p.pos.z.sub(at.z)).flat_len();
        if near.raw() > t::tear_radius().mul(Fx::ratio(1, 2)).raw() {
            return;
        }
        effect.take_hit(1, owner);
        let radius = t::tear_radius();
        for i in 0..MAX_PLAYERS {
            if !self.effects_reach(i, effect.owner) || !self.inside(i, at, radius) {
                continue;
            }
            let q = self.players[i];
            let away = V3::new(q.pos.x.sub(at.x), Fx::ZERO, q.pos.z.sub(at.z)).normalized();
            let (guarding, parried) = guard_against(&q, at, false);
            apply_hit(
                &mut self.players[i],
                Hit {
                    damage: t::tear_damage(),
                    hitstun: effect.source().hitstun,
                    blockstun: effect.source().blockstun,
                    knockback: effect.source().knockback,
                    launch: t::tear_launch(),
                    grabs: 0,
                    by: effect.owner,
                    dir: away,
                    blocked: guarding,
                    parried,
                    interrupts: true,
                },
            );
            let hold = impact_freeze(effect.source().hitstop, guarding && !parried);
            freeze(&mut self.players[i], hold);
            freeze(&mut self.players[owner], hold);
        }
        let share = Fx::ratio(t::tear_damage(), effect.damage().max(1));
        for slot in 0..MAX_MONSTERS {
            self.gore_for_a_share(slot, effect, 1, at, radius, share);
        }
        self.gore_critters(effect, at, radius, share);
        // Torn out: gone, in his hands, and the run after it is over.
        effect.life = effect.age;
        if let Mechanic::Forms { rush, .. } = &mut self.players[owner].mechanic {
            *rush = 0;
        }
        let me = &mut self.players[owner];
        me.vel.x = Fx::ZERO;
        me.vel.z = Fx::ZERO;
    }

    /// **The shoulder** (exploration 0015, C1): a Champion's Rush that runs
    /// into a body -- a fighter, or a creature -- hits it, once a run, and the
    /// hit is **the string's first**. The next click throws the second link,
    /// so the core combo starts from the move that crossed the gap rather than
    /// after it.
    ///
    /// Only a bare Rush shoulders: a Rush move thrown out of it is its own
    /// blow. On contact the run stops where it struck. Blocked, it is a
    /// blocked hit and opens nothing; parried, he is staggered like any
    /// parried swing.
    fn shoulder(&mut self, i: usize) {
        let p = self.players[i];
        let Mechanic::Forms {
            rush,
            rush_vel,
            fury,
            ..
        } = p.mechanic
        else {
            return;
        };
        if rush == 0 || fury.shouldered || !p.action.actionable() || p.health <= 0 {
            return;
        }
        let ahead = V3::new(rush_vel.x, Fx::ZERO, rush_vel.z).normalized();
        let reach = t::body_radius()
            .add(t::body_radius())
            .add(t::shoulder_reach());
        let mut struck = false;
        let mut blocked = false;
        // The first body in its way, in seat order: a shoulder hits once.
        let met = (0..self.present()).find(|&other| {
            let q = &self.players[other];
            let d = q.pos.sub(p.pos);
            other != i
                && q.health > 0
                && d.y.abs().raw() < t::body_height().raw()
                && d.flat_len().raw() <= reach.raw()
                && ahead.dot(V3::new(d.x, Fx::ZERO, d.z)).raw() > 0
                && !q.action.invulnerable()
        });
        if let Some(other) = met.filter(|&other| self.foes(i, other)) {
            let q = self.players[other];
            {
                let (guarding, parried) = guard_against(&q, p.pos, false);
                apply_hit(
                    &mut self.players[other],
                    Hit {
                        damage: t::shoulder_damage(),
                        hitstun: t::shoulder_hitstun(),
                        blockstun: t::shoulder_blockstun(),
                        knockback: t::shoulder_knockback(),
                        launch: Fx::ZERO,
                        grabs: 0,
                        by: i as u8,
                        dir: ahead,
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
                    self.players[other].parried = PARRY_FLOURISH;
                }
                let hold = impact_freeze(t::shoulder_freeze(), guarding && !parried);
                freeze(&mut self.players[other], hold);
                freeze(&mut self.players[i], hold);
                struck = true;
                blocked = guarding || parried;
            }
        }
        if !struck {
            let at = p.pos.add(ahead.scale(t::body_radius()));
            for slot in 0..MAX_MONSTERS {
                let Some(mut beast) = self.monsters[slot] else {
                    continue;
                };
                if !beast.alive() {
                    continue;
                }
                let Some(part) =
                    beast.part_struck(at, reach.sub(t::body_radius()), p.hurt_height())
                else {
                    continue;
                };
                let (_, guarded) = beast.take_blow(
                    part,
                    t::shoulder_damage(),
                    &monster::Blow {
                        from: p.pos,
                        unblockable: false,
                        who: i as u8,
                        class: p.class,
                        kind: monster::Blow::NO_MOVE,
                    },
                );
                self.monsters[slot] = Some(beast);
                // The animal does not stop for one fighter; he does, a beat,
                // which is the shoulder landing.
                freeze(&mut self.players[i], t::shoulder_freeze());
                struck = true;
                blocked = guarded.turned();
                break;
            }
        }
        if !struck {
            return;
        }
        let me = &mut self.players[i];
        me.vel.x = Fx::ZERO;
        me.vel.z = Fx::ZERO;
        if let Mechanic::Forms {
            rush,
            fury,
            chain,
            chain_left,
            chain_hit,
            ..
        } = &mut me.mechanic
        {
            *rush = 0;
            fury.shouldered = true;
            if !blocked {
                *chain = 1;
                *chain_left = t::chain_grace();
                *chain_hit = true;
            }
        }
        if !blocked {
            feed_the_momentum(me, None);
        }
    }
}

/// **Break** (exploration 0015, C6): what a Champion's blow that landed
/// `dealt` on a creature adds to its strain over the damage -- a hammer, a
/// third link, a true link and a full charge each weigh more than they cut,
/// the heaviest of them counting -- and a full charge's extra goes some way
/// to its poise as well. The Meteor's crater knocks it off its feet outright.
/// He is the class that puts a creature on the floor.
pub(crate) fn impact(beast: &mut Monster, p: &Player, kind: u8, dealt: i32) {
    use moves::champion as c;
    if p.class != Class::Champion || dealt <= 0 {
        return;
    }
    let true_link = matches!(p.mechanic, Mechanic::Forms { fury, .. } if fury.true_link);
    let full = c::is_full(kind);
    let mut weight = 100;
    if c::weapon(kind) == c::HAMMER {
        weight = weight.max(t::impact_hammer());
    }
    if c::in_string(kind) && place_in_string(p, kind) == Some(c::DEPTH - 1) {
        weight = weight.max(t::impact_third());
    }
    if true_link {
        weight = weight.max(t::impact_beat());
    }
    if full {
        weight = weight.max(t::impact_full());
    }
    let extra = dealt * (weight - 100).max(0) / 100;
    // A finisher weighs more for the momentum it spent, up to twice as much
    // again for a full meter -- and a full meter spent knocks it off its feet
    // (exploration 0015, the fourth round).
    let spent = match p.mechanic {
        Mechanic::Forms { fury, .. } if full => fury.spent.min(100) as i32,
        _ => 0,
    };
    let extra = extra * (100 + 2 * spent) / 100;
    let poise = if full {
        extra * t::impact_poise() / 100
    } else {
        0
    };
    beast.take_weight(extra, poise);
    if spent >= 100 {
        beast.stumble_for(t::meteor_stumble());
    }
}

/// **Is this a blow that clashes** (exploration 0015, C7): a Champion in
/// the active frames of a full charge or of the hammer's third link, or in a
/// Rush's last frames with the shoulder down. Meeting a swing with one of
/// these cancels the swing rather than taking it.
fn clashing(p: &Player) -> bool {
    use moves::champion as c;
    if p.class != Class::Champion {
        return false;
    }
    let heavy = matches!(
        p.action,
        Action::Active { kind, .. } if c::is_full(kind) || kind == c::EARTHBREAKER
    );
    let shoulder_down = match p.mechanic {
        Mechanic::Forms { rush, fury, .. } => {
            rush > 0 && rush <= t::rush_eat_frames() && !fury.eaten && p.action.actionable()
        }
        _ => false,
    };
    heavy || shoulder_down
}

/// **Feed the meter** (exploration 0015, C4): a blow of his landed. Every
/// one is worth `momentum_hit`; one of a different weapon from the last more,
/// a true link more, a full charge more. `kind` is the move that landed, or
/// `None` for the shoulder, which is no weapon.
fn feed_the_momentum(p: &mut Player, kind: Option<u8>) {
    use moves::champion as c;
    if p.class != Class::Champion {
        return;
    }
    let Mechanic::Forms { fury, .. } = &mut p.mechanic else {
        return;
    };
    let mut gain = t::momentum_hit();
    if let Some(kind) = kind {
        let weapon = c::weapon(kind);
        if fury.last_weapon != 0 && fury.last_weapon - 1 != weapon {
            gain += t::momentum_swap();
        }
        if fury.true_link {
            gain += t::momentum_beat();
        }
        if c::is_full(kind) {
            gain += t::momentum_full();
        }
        fury.last_weapon = weapon + 1;
    }
    let max = t::momentum_max().clamp(1, u16::MAX as i32);
    fury.momentum = (fury.momentum as i32 + gain).clamp(0, max) as u16;
    fury.momentum_left = t::momentum_holds().min(u8::MAX as u16) as u8;
}

/// A blow taken costs a share of the meter.
fn hurt_the_momentum(p: &mut Player) {
    if let Mechanic::Forms { fury, .. } = &mut p.mechanic {
        let lost = fury.momentum as i32 * t::momentum_hurt().clamp(0, 100) / 100;
        fury.momentum = (fury.momentum as i32 - lost).max(0) as u16;
    }
}

/// Where in the string a move of his stands: a link's own place, and for an
/// aerial, a takeoff or a finisher the place he remembered when it began.
/// `None` for anything outside the string.
fn place_in_string(p: &Player, kind: u8) -> Option<u8> {
    use moves::champion as c;
    if let Some(link) = c::link_of(kind) {
        return Some(link);
    }
    match p.mechanic {
        Mechanic::Forms { fury, .. } if c::in_string(kind) || c::is_full(kind) => Some(fury.place),
        _ => None,
    }
}

/// Is this Champion holding a link at the top of its wind-up?
fn holding(p: &Player) -> bool {
    p.class == Class::Champion
        && p.action
            .channelling()
            .is_some_and(|(kind, _)| moves::charge(p.class, kind) == Some(moves::Charge::Heft))
}

/// Is this Champion winding up a move he may still hold to a finisher --
/// its button down since the press, in its startup or past it?
fn winding_a_hold(p: &Player) -> bool {
    may_hold(p)
        && match p.action {
            Action::Startup { kind, .. } => moves::charge(p.class, kind).is_some(),
            _ => holding(p),
        }
}

/// Was the link running now started by a fresh press of its button?
fn may_hold(p: &Player) -> bool {
    matches!(p.mechanic, Mechanic::Forms { fury, .. } if fury.may_hold)
}

/// The Blood mage's tap `kind` held into its field: the field's cost paid,
/// and its hold begun.
fn into_the_field(p: &mut Player, kind: u8) -> Action {
    let field = moves::blood::held_into(kind).unwrap_or(kind);
    pay(p, moves::get(p.class, field).cost);
    Action::Channel {
        kind: field,
        held: 0,
    }
}

/// The link `kind` held to a finish: its weapon's finisher for that place in
/// the string, aimed now and wound up from its own startup. The link itself if
/// it has none.
fn finish_with(p: &mut Player, who: usize, kind: u8, input: Input, scene: &Scene) -> Action {
    use moves::champion as c;
    let Some(place) = place_in_string(p, kind).filter(|_| c::holdable(kind)) else {
        return Action::Active {
            kind,
            left: moves::get(p.class, kind).active,
        };
    };
    let big = c::finisher(c::weapon(kind), place);
    lock_aim(p, who, big, input, scene);
    begin_full_charge(p, big, place);
    Action::Startup {
        kind: big,
        left: moves::get(p.class, big).startup,
    }
}

/// A full charge has begun, from the link at `place` in the string. Its own
/// move from here: the weapon is its weapon, the string stands where the link
/// it came from would have left it, and the Meteor leaps for where the
/// crosshair was.
fn begin_full_charge(p: &mut Player, kind: u8, place: u8) {
    use moves::champion as c;
    if let Mechanic::Forms {
        form,
        chain,
        chain_left,
        chain_hit,
        fury,
        liquid,
        ..
    } = &mut p.mechanic
    {
        let from = *form;
        *form = Form::of_move(kind);
        liquid.call(from);
        // A finisher finishes: the string is over, and the next press opens
        // one. It stands at its link's place for the refund.
        *chain = 0;
        *chain_left = 0;
        *chain_hit = false;
        fury.true_link = false;
        fury.place = place;
        // **It spends the meter**, whatever is in it, and is worth what it
        // spent (`momentum_mul`, `stagger_by_the_spend`).
        let max = t::momentum_max().max(1);
        fury.spent = (fury.momentum as i32 * 100 / max).clamp(0, 100) as u8;
        fury.momentum = 0;
        fury.momentum_left = 0;
    }
    p.channelled = Fx::ZERO;
    // **From the air** (2026-10-13, from play) a finisher keeps whatever the
    // jump was carrying -- the Crescent off a leap goes on with him -- and
    // hangs on nothing. The hammer's two are slammed straight back to the
    // floor instead, down the crosshair's line held inside a cone round
    // straight down, and break it where he lands (`lands_on_the_floor`).
    if !p.grounded {
        p.air_stall = 0;
        if kind == c::SHOCKWAVE || kind == c::METEOR {
            p.vel = aim::plunge_dir(p.aim_dir(), t::slam_cone()).scale(t::slam_dive());
        }
        return;
    }
    if kind == c::METEOR {
        // Up at the knob's rise and down under gravity: the time in the air
        // is what the rise buys, and the run across is spread over it so he
        // comes down where the crosshair was.
        let to = p.aim_at();
        let air = t::meteor_rise()
            .add(t::meteor_rise())
            .div(t::gravity().abs().max(Fx::ONE));
        let across = V3::new(to.x.sub(p.pos.x), Fx::ZERO, to.z.sub(p.pos.z));
        p.vel = V3::new(across.x.div(air), t::meteor_rise(), across.z.div(air));
        p.grounded = false;
    }
}

/// **What momentum is worth to a blow** (exploration 0015, the fourth
/// round): every hit he lands is worth its share of `momentum_buff` for the
/// meter he carries, and a finisher -- which carries none, having spent it --
/// its share of `momentum_spend` for what it spent. One for anybody else.
fn momentum_mul(p: &Player, kind: u8) -> Fx {
    Fx::ONE.add(carried_share(p)).add(spent_share_of(p, kind))
}

/// The meter he carries, as a share of a blow: `momentum_buff` at full.
fn carried_share(p: &Player) -> Fx {
    let Mechanic::Forms { fury, .. } = p.mechanic else {
        return Fx::ZERO;
    };
    let max = t::momentum_max().max(1);
    Fx::ratio(fury.momentum as i32 * t::momentum_buff(), max * 100)
}

/// What a finisher spent, as a share of its blow; nothing for anything else.
fn spent_share_of(p: &Player, kind: u8) -> Fx {
    match p.mechanic {
        Mechanic::Forms { fury, .. } if moves::champion::is_full(kind) => spend_share(fury.spent),
        _ => Fx::ZERO,
    }
}

/// What a finisher that spent `spent` per cent of a full meter adds, as a
/// share: `momentum_spend` of it at a full meter.
fn spend_share(spent: u8) -> Fx {
    Fx::ratio(spent.min(100) as i32 * t::momentum_spend(), 100 * 100)
}

/// **The stagger a spent meter buys** (exploration 0015, the fourth round):
/// a finisher that lands on a fighter leaves them staggered for its own stun
/// and its share of `momentum_stagger` more -- more and more obvious the more
/// it spent. A full meter spent knocks them down outright.
fn stagger_by_the_spend(victim: &mut Player, base: u16, spent: u8) {
    if spent == 0 || victim.action.invulnerable() {
        return;
    }
    let extra = (t::momentum_stagger() as u32 * spent.min(100) as u32 / 100) as u16;
    let left = if spent >= 100 {
        base.max(t::knockdown_frames()).saturating_add(extra)
    } else {
        base.saturating_add(extra)
    };
    victim.action = Action::Stagger { left };
    victim.stun_total = left;
}

/// What a true link multiplies its damage by: one for anything else.
fn true_link_mul(p: &Player) -> Fx {
    match p.mechanic {
        Mechanic::Forms { fury, .. } if fury.true_link => {
            Fx::ONE.add(Fx::ratio(t::true_link_damage() as i32, 100))
        }
        _ => Fx::ONE,
    }
}

/// The frames a true link adds to its freeze: none for anything else.
fn true_link_freeze(p: &Player) -> u16 {
    match p.mechanic {
        Mechanic::Forms { fury, .. } if fury.true_link => t::true_link_freeze(),
        _ => 0,
    }
}

/// The Champion's three weapon buttons, as bits in button order.
fn weapon_bits(input: Input) -> u8 {
    (input.has(Input::LEFT) as u8)
        | (input.has(Input::MIDDLE) as u8) << 1
        | (input.has(Input::RIGHT) as u8) << 2
}

/// Read the weapon buttons: the presses kept for him (`Fury::fresh`), a hold
/// let go, and a press made before the Beat. Off the raw input, before an
/// impact freeze swallows it.
fn find_the_beat(p: &mut Player, input: Input) {
    let winding = match p.action {
        Action::Startup { kind, .. } | Action::Channel { kind, .. } => Some(kind),
        _ => None,
    };
    let clock = beat_clock(p);
    let in_string = champion_chain(p).0 > 0;
    let early = in_string && clock < t::beat_from();
    let on_beat = in_string && clock >= t::beat_from() && clock < t::beat_to();
    let Mechanic::Forms { fury, .. } = &mut p.mechanic else {
        return;
    };
    let now = weapon_bits(input);
    let fresh = now & !fury.clicks;
    fury.clicks = now;
    // **Only a press throws a link** (2026-10-20, from play): a press is kept
    // for `press_kept` frames whether or not the button is still down, and
    // then forgotten. Holding the button is not pressing it again.
    if fresh != 0 {
        fury.fresh |= fresh;
        fury.fresh_left = t::press_kept().min(u8::MAX as u16) as u8;
    } else if fury.fresh_left > 0 {
        fury.fresh_left -= 1;
    }
    if fury.fresh_left == 0 {
        fury.fresh = 0;
    }
    // **A hold is one press, held** (2026-10-13, from play): let go of the
    // button a link was pressed for and that link can no longer be held, so
    // the next click of a string -- pressed during this one's wind-up -- is
    // never read as this one held long. It used to be: clicking fast, a
    // second press down at the top of a hammer's wind-up was a finisher.
    if let Some(kind) = winding {
        if now & 1 << moves::champion::weapon(kind) == 0 {
            fury.may_hold = false;
        }
    }
    // **The Beat is judged on the click**, the one thing the player times:
    // the gap's first press in the sweet spot makes the link it throws a
    // true link, even if that link comes out a frame or two after. A press
    // before it spends it for the gap -- the link it throws, or the next if
    // it was lost, is ordinary -- so mashing never finds it.
    if fresh != 0 && early {
        fury.early = true;
    }
    if fresh != 0 && on_beat && !fury.early {
        fury.on_beat = true;
    }
}

/// **The Beat's clock** (2026-10-20, from play): frames since he could act
/// again in the gap between two links of his string -- nought while a link
/// is still running and owes its recovery, then counting up. The string's
/// window counts down on the same frames (`tuning::chain_grace`), so this is
/// how far along it he is. Nought outside a string.
///
/// **The Beat** (exploration 0015, C2; a rhythm since 2026-10-20): the
/// gap's first press, made `tuning::beat_from` to `beat_to` frames along --
/// the same frames for every weapon -- throws a **true link**: it hits
/// harder and sticks longer. It used to be a press inside the last link's
/// impact freeze, which is as soon as possible, and mashing found it.
/// Carried through the third hit into the next string's first.
fn beat_clock(p: &Player) -> u16 {
    match champion_chain(p) {
        (0, _) => 0,
        (_, left) => t::chain_grace().saturating_sub(left),
    }
}

/// Drive a Champion `reach` along `dir` at `speed`, on the Rush's own
/// machinery: the run holds its line, is braced, and ends on a hit like a
/// Rush. It spends no charge -- the full charge already paid for it.
fn drive_the_champion(p: &mut Player, dir: V3, reach: Fx, speed: Fx) {
    let frames = reach.div(speed.max(Fx::ONE)).div(DT).to_int().clamp(1, 240) as u16;
    if let Mechanic::Forms {
        rush,
        rush_vel,
        fury,
        ..
    } = &mut p.mechanic
    {
        *rush = frames;
        *rush_vel = V3::new(dir.x.mul(speed), Fx::ZERO, dir.z.mul(speed));
        fury.braced = 0;
        fury.eaten = false;
        fury.shouldered = true;
    }
}

/// Does `hit` meet a Champion's heavy blow? If so the blow is spent on it:
/// he freezes on the impact, flourishes, is fed for it, and remembers who
/// threw it for `World::settle_the_clashes`. Blocks, parries, blows that take
/// no frames and blows thrown by nobody (the world's own) do not clash.
fn meets_a_clash(defender: &mut Player, hit: &Hit) -> bool {
    if hit.blocked || hit.parried || !hit.interrupts || hit.by == NOBODY || !clashing(defender) {
        return false;
    }
    if let Mechanic::Forms { fury, rush, .. } = &mut defender.mechanic {
        fury.clash_by = hit.by;
        // A Rush's shoulder, once a run, as its eaten blow is.
        if *rush > 0 {
            fury.eaten = true;
        }
    }
    freeze(defender, t::clash_freeze());
    defender.parried = PARRY_FLOURISH;
    feed_the_momentum(defender, None);
    true
}

/// **The Rush, braced** (exploration 0015, C1): what a blow does to a
/// Champion who is running.
///
/// Three bands, and nothing outlasts the run, so there is no soak to sit in:
///
/// * **the last frames** (`tuning::rush_eat_frames`) eat one blow whole, once
///   a Rush -- the shoulder down as he arrives, a timing rather than a buff;
/// * **the rest of the run** takes a share of the damage
///   (`tuning::rush_damage_taken`) and keeps going -- no stun, no shove;
/// * **too much**: the shares are summed over the run, and the blow that
///   takes the sum past `tuning::rush_breaks` lands the ordinary way, at its
///   braced share, and ends the Rush as any hit does.
///
/// `None` when the blow is spent here; the hit to apply otherwise. Blocks,
/// parries and blows that take no frames are not his to brace against.
fn brace_the_rush(defender: &mut Player, hit: Hit) -> Option<Hit> {
    if hit.blocked || hit.parried || !hit.interrupts {
        return Some(hit);
    }
    let Mechanic::Forms { rush, fury, .. } = &mut defender.mechanic else {
        return Some(hit);
    };
    if *rush == 0 {
        return Some(hit);
    }
    if *rush <= t::rush_eat_frames() && !fury.eaten {
        fury.eaten = true;
        return None;
    }
    let taken = (hit.damage.max(0) as i64 * t::rush_damage_taken() as i64 / 100) as i32;
    let braced = fury.braced as i32 + taken;
    fury.braced = braced.min(u16::MAX as i32) as u16;
    if braced > t::rush_breaks() {
        return Some(Hit {
            damage: taken,
            ..hit
        });
    }
    defender.wound(taken);
    None
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
fn countdown(p: &mut Player, want_guard: bool, input: Input) -> Option<Action> {
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
        // **The startup is the tap window.** A Strike's button still down on
        // the last frame of the startup is a hold, and the hold sits between
        // the startup and the active frames: she has finished winding up and
        // is now gathering, at the crawl, with the aim live. Released inside
        // the startup, this arm is never reached and the pillar is the pillar
        // as built. See `moves::Charge::Strike` and `step_channel`.
        // **A held click is the field** (2026-10-13): right click's Command
        // becomes the Carpet, left click's Haemorrhage the Hemoplague. Not a
        // charge -- the tap is the tap, and holding past it is a different
        // move, whose cost is paid where it begins. **Held is the Champion's
        // window** (2026-10-14), `tuning::hold_from` counted from the press:
        // Command winds up in a frame, and a tap that had to come up inside
        // it was hardly a tap. A wind-up shorter than the window waits at
        // its top for the press to decide (`step_channel`).
        Action::Startup { kind, left: 0 }
            if p.class == Class::BloodMage
                && moves::blood::held_into(kind).is_some()
                && input.has(moves::blood::click_of(kind)) =>
        {
            if moves::get(p.class, kind).startup + 1 < t::hold_from() {
                Action::Channel { kind, held: 0 }
            } else {
                into_the_field(p, kind)
            }
        }
        Action::Startup { kind, .. }
            if moves::charge(p.class, kind) == Some(moves::Charge::Strike)
                && moves::get(p.class, kind).channel > 0
                && input.has(channel_button(p.class, kind)) =>
        {
            Action::Channel { kind, held: 0 }
        }
        // **Tap to combo, hold to finish** (exploration 0015, the fourth
        // round), the same arm: a link whose button was pressed for it and is
        // still down at the top of its wind-up waits there for the press to be
        // long enough to mean it -- `tuning::hold_from`, counted from the press
        // -- and then is that weapon's finisher. A wind-up as long as that
        // decides on the next frame. See `step_channel`.
        Action::Startup { kind, .. }
            if moves::charge(p.class, kind) == Some(moves::Charge::Heft)
                && may_hold(p)
                && input.has(channel_button(p.class, kind)) =>
        {
            Action::Channel { kind, held: 0 }
        }
        Action::Startup { kind, .. } => Action::Active {
            kind,
            left: moves::get(p.class, kind).active,
        },
        Action::Active { kind, left } if left > 0 => Action::Active {
            kind,
            left: left - 1,
        },
        Action::Active { kind, .. } => {
            // Slam's weight goes with its active frames, landed or not.
            bulwark::spend_on_slam(p, kind);
            Action::Recovery {
                kind,
                left: moves::get(p.class, kind).recovery,
            }
        }
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
            p.taken = 0;
            // Let go, and whatever the grab banked is spent now. See
            // `Player::thrown`.
            if p.thrown.raw() > 0 {
                p.vel.y = p.thrown;
                p.grounded = false;
            }
            p.thrown = Fx::ZERO;
            Action::Free
        }
        Action::BlockStun { .. } | Action::HitStun { .. } | Action::Stagger { .. } => {
            // Free to act again: the string is over, and the swell is whole
            // for whatever lands next.
            p.taken = 0;
            Action::Free
        }
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
    herd: &Herd,
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
    // A spent click counts again once it has come up (`Player::spent_clicks`).
    p.spent_clicks &= input.bits;
    // **What the floor does to the feet** (bestiary P4): a dodge from inside
    // tar carries half as far, a jump from inside web hardly leaves it. The
    // walk is slowed through the ordinary slow (`World::feel_the_floor`);
    // these two are read here, where the dodge and the jump are thrown. On
    // clear floor all three are one, and multiplying by one changes nothing.
    let foot = if p.footed() {
        scene
            .arena
            .floor
            .underfoot(p.pos, hazard::reach::FIGHTERS, true, 0)
    } else {
        hazard::Underfoot::CLEAR
    };
    // Before anything moves the body, so the frame the feet land on still
    // sees the fall that got them there.
    bulwark::track_fall(p);
    // **The wall, as the last resolve left it** (`cling_to_the_wall`), read
    // before anything moves and then aged a frame: the resolve below puts it
    // back to zero if the body is still against one. Aged up here, above the
    // rider's early returns, so a wall touched before mounting is not a wall
    // the dismount can jump off.
    if p.footed() {
        p.wall_since = u8::MAX;
        p.wall_jumped = Wall::NONE;
        p.wall_jumps = 0;
        p.wall_press = 0;
        p.sliding = false;
        p.slide_flip = 0;
        p.ledge_regrab = 0;
    }
    p.ledge_regrab = p.ledge_regrab.saturating_sub(1);
    let on_wall = p.wall_since == 0;
    let near_wall = p.wall_since <= t::wall_jump_grace();
    p.wall_since = p.wall_since.saturating_add(1);
    // Standing on the creature is a different tick: no gravity, no arena, and
    // movement that happens in the animal's frame rather than the world's.
    // **The dead do not ride.** A fighter killed on its back falls off it,
    // rather than being carried round the arena for the rest of the hunt.
    // **On a critter's back** (the Hornback's cow): its own, smaller tick --
    // see `step_critter_rider`. The dead, and a back that has gone, are
    // fallen off.
    if let Some(i) = critter::ridden(p.mount) {
        let crowd = scene.critters;
        let back = crowd.get(i).copied().filter(|c| c.rideable(crowd.sp()));
        match back {
            Some(c) if p.health > 0 => {
                return step_critter_rider(p, who, input, &c, crowd.sp(), scene, &out);
            }
            _ => {
                p.mount = monster::NO_MOUNT;
                p.grounded = false;
            }
        }
    }
    if p.aboard() && p.health <= 0 {
        if let Some(beast) = ridden(p, herd) {
            p.pos = beast.world_of(mount_part(p.mount), p.local);
        }
        p.mount = monster::NO_MOUNT;
        p.grounded = false;
    }
    if p.aboard() {
        match ridden(p, herd) {
            Some(beast) => return step_rider(p, who, input, beast, scene, &out),
            // The creature is gone. Whatever you were standing on is not there
            // any more, so neither are you.
            None => p.mount = monster::NO_MOUNT,
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
    if !input.has(Input::SHIFT) {
        shadow::shift_spent(p, false);
    }
    p.space_held = input.has(Input::SPACE);
    // The two keys the v2 grammar added, read on the press like the mechanic.
    let pressed_f = input.has(Input::KEY_F) && !p.f_held;
    p.f_held = input.has(Input::KEY_F);
    let pressed_r = input.has(Input::KEY_R) && !p.r_held;
    p.r_held = input.has(Input::KEY_R);
    let pressed_side_b = input.has(Input::SIDE_B) && !p.side_b_held;
    p.side_b_held = input.has(Input::SIDE_B);
    let pressed_earth = input.has(moves::elementalist::keys::EARTH) && !p.earth_held;
    p.earth_held = input.has(moves::elementalist::keys::EARTH);

    // **Holding a ledge is a tick of its own** (`hang_on`), like riding the
    // creature: no gravity, no resolve, no moves -- the stick climbs, lets go
    // or goes hand over hand. Below the press edges, so a button held through
    // the hang is not pressed again on the frame it ends. **A blow takes the
    // hands off it** (`apply_hit` has already set the stun and the shove), and
    // so does dying.
    if p.ledge.is_some() {
        if p.action.actionable() && p.health > 0 {
            step_mechanic(p, scene.arena);
            hang_on(p, input, scene.arena);
            return;
        }
        p.ledge = Ledge::NONE;
        p.ledge_regrab = t::ledge_regrab();
    }

    // **The Blood mage under her ground, and her Sweep strung** (2026-10-18).
    swim_or_surface(p, input);
    string_the_sweep(p, input);

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
    if pressed_space {
        shadow::bank_dash_jump(p);
        bank_the_flurry_jump(p);
    }
    step_mechanic(p, scene.arena);
    hold_the_churn(p, input);

    let want_guard = input.has(Input::RIGHT) && p.shield().is_some_and(|sh| sh.in_hand());
    let (ax, az) = input.move_axis();
    // Crouch is a stance, not an action: it holds while the key is down and
    // only while you are otherwise free to move.
    p.crouching = input.has(Input::CROUCH) && p.grounded && p.action.actionable();

    // **Rush is the universal cancel.** Spent from a recovery it ends the
    // recovery, which is the Champion's whole answer to having committed to the
    // wrong move -- and, more often, what turns two moves into a combo. One
    // charge means that is always a decision. See `docs/design/champion.md`.
    if pressed_mechanic
        && (matches!(p.action, Action::Recovery { .. }) || holding(p))
        && start_rush(p, who, input, scene)
    {
        // A Rush also takes back a hold (exploration 0015, C3): a held swing
        // is a threat, and a threat you can walk away from is a feint.
        p.action = Action::Free;
    }

    // The jump button holds the takeoff window open, and a connected chain link
    // lets the next one start early. Both are Champion-only and both are no-ops
    // for everybody else; both go here, before the countdown, for the same
    // reason the Rush cancel does -- a recovery that has been cut short has to
    // reach the input below on the frame it was cut, not the frame after.
    arm_takeoff(p, input);
    rise_window(p, input, field);
    // A stored charge is let go by a fresh press of its click, so the click
    // that charged it has to come up first.
    if p.vessel != NO_VESSEL && !input.has(moves::blood::click_of(p.vessel)) {
        p.vessel_ready = true;
    }
    rise_out_of_a_swing(p, who, pressed_space, input, scene);
    refresh_the_rise(p);
    bank_the_leap(p, pressed_space);
    chain_cancel(p);
    cancel_the_carpet(
        p,
        input,
        &out,
        [pressed_mechanic, pressed_f, pressed_r, pressed_side_b],
    );
    cancel_into_a_push(p, input, &out, pressed_mechanic);
    jump_off_the_crack(p, input, scene);

    queue_the_shadow(p, input);
    follow_the_shadow(p, input);

    // **A swing out of the Reaver's carry.** The strike on arrival: it cuts the
    // dodge's tail the way the jump out of it does, before the countdown, so
    // the swing comes out on this frame rather than a slide's length later.
    // See `shadow::swing_out_of_the_carry`.
    shadow::swing_out_of_the_carry(p, input);
    // The Dual mage's hop out of a finished auto, before the countdown so the
    // recovery it shortens is the one counted down this frame.
    hop_after_the_auto(p, input, scene);
    dash_down_the_flurry(p, scene);

    // A channel resolves **instead of** the countdown, because it is the one
    // action whose next state depends on a button and on the scene rather than
    // on a number running down: held, it winds on and re-aims; released, it
    // throws the move it was winding. Falling through to the input below would
    // read the same press again and buy the same ability twice.
    if let Some((kind, held)) = p.action.channelling() {
        p.action = step_channel(p, who, kind, held, input, scene);
    } else {
        p.action = match countdown(p, want_guard, input) {
            Some(next) => next,
            None => {
                // Clicks are checked before the dodge, which is what disambiguates
                // shift. Shift with a click is the stronger version of that attack;
                // shift with only a direction is a dodge. See controls.md.
                //
                // The Champion has no special: its identity is the three weapons on
                // the three clicks, and `Q` is free.
                if let Some(kind) = input
                    .has(Input::SPECIAL)
                    .then(|| keyed_q(p))
                    .flatten()
                    .filter(|k| p.can_throw(*k, &out))
                {
                    begin_move(p, who, kind, input, scene, true)
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
                // `F`: a move about her own body, and which one is where her
                // feet are. See `keyed_f`.
                else if let Some(kind) = pressed_f
                    .then(|| keyed_f(p))
                    .flatten()
                    .filter(|k| p.can_throw(*k, &out))
                {
                    begin_move(p, who, kind, input, scene, true)
                }
                // `R`: Quake on her own feet. The second side button: Quake
                // where the crosshair is. See `keyed_r` and `keyed_side_b`.
                else if let Some(kind) = pressed_r
                    .then(|| keyed_r(p))
                    .flatten()
                    .filter(|k| p.can_throw(*k, &out))
                {
                    begin_move(p, who, kind, input, scene, true)
                } else if let Some(kind) = pressed_side_b
                    .then(|| keyed_side_b(p))
                    .flatten()
                    .filter(|k| p.can_throw(*k, &out))
                {
                    begin_move(p, who, kind, input, scene, true)
                }
                // Which move a click asks for. After the mechanic, so that Rush
                // can be started while a click is held down, and before the dodge,
                // because a click is what disambiguates shift.
                else if let Some(kind) = clicked_move(p, input).filter(|k| p.can_throw(*k, &out))
                {
                    // A no-op for anybody who is not the Champion; the weapon in
                    // hand and the dash underneath it are that class's alone.
                    rise_from_the_floor(p, kind, scene);
                    back_to_the_floor(p, kind, scene);
                    spend_the_rise(p, kind);
                    begin_champion(p, kind);
                    begin_move(p, who, kind, input, scene, true)
                }
                // The Elementalist's Raise: an instant on the earth click's
                // press, the way it was on the mechanic key. See
                // `raise_on_the_click`.
                else if raise_on_the_click(p, pressed_earth) {
                    mechanic_action(p, who, input, scene);
                    Action::Free
                } else if dives_in(p, who, input, scene) {
                    // **The Blood mage sinks into her ground** (2026-10-18):
                    // `shift` on it, with no stain under the crosshair for
                    // the teleport, is the swim rather than the dodge.
                    p.swim = SWIM_UNDER;
                    Action::Free
                } else if input.has(Input::SHIFT)
                    && (ax != 0 || az != 0)
                    && p.swim == SWIM_NONE
                    && !shadow::shift_spent(p, true)
                {
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
                    if !dual::may_dodge(p) {
                        // Ascending, the Dual mage has no dodge: she flies
                        // instead, and loss of control is loss of the option
                        // to decline. See `docs/design/dual-mage.md`.
                        Action::Free
                    } else if let Some(pool) = (may_commit || p.class == Class::BloodMage)
                        .then(|| pool_under_the_crosshair(p, who, input, scene))
                        .flatten()
                    {
                        // The Blood mage's blink: a dodge thrown with the
                        // reticle on one of her pools puts her in it, and the
                        // pool is spent. The Reaver's dash pointed at the
                        // class's object instead of at a body -- and with
                        // nothing to cross, the crossing is instant. The
                        // airborne one spends the airdodge, like hers -- and is
                        // there even once it is spent, since a stain is on the
                        // floor and so is she when she arrives: the zip to the
                        // blood she sent from a jump (2026-10-12).
                        if !p.grounded {
                            p.air_dodged = true;
                        }
                        p.pos = scene.effects[pool].map_or(p.pos, |e| e.pos);
                        p.vel = V3::ZERO;
                        p.blinked = pool as u16;
                        Action::Dodge {
                            left: t::dodge_frames(),
                        }
                    } else if let Some(slot) = may_commit
                        .then(|| stone_under_the_crosshair(p, who, input, dir, scene))
                        .flatten()
                    {
                        // **The Elementalist's dodge into a stone breaks
                        // through it.** An ordinary dodge along the stick --
                        // same frames, same invulnerability -- and the stone
                        // is gone as she passes, its slot freed and broken
                        // ground left where it stood: whoever was chasing her
                        // has a scar between them and her. `advance` spends
                        // the mark on this frame. In the air it is the
                        // airdodge, and costs the airdodge.
                        p.breaking = slot as u8;
                        if p.grounded {
                            p.vel.x = dir.x.mul(t::dodge_speed().mul(foot.dodge));
                            p.vel.z = dir.z.mul(t::dodge_speed().mul(foot.dodge));
                            Action::Dodge {
                                left: t::dodge_frames(),
                            }
                        } else {
                            p.air_dodged = true;
                            p.vel.x = dir.x.mul(t::air_dodge_speed());
                            p.vel.z = dir.z.mul(t::air_dodge_speed());
                            p.vel.y = Fx::ZERO;
                            Action::Dodge {
                                left: t::air_dodge_frames(),
                            }
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
                    } else if may_commit && dual::may_blink(p) {
                        // The Dual mage's dodge at the first tier is a **blink**:
                        // the same commitment, the same invulnerable window and
                        // the same tail, with the travelling taken out. She is
                        // put where the dodge would have ended, on its first
                        // frame, or against the first stone or wall in the way
                        // -- `aim::blink_to` decides which. In the air it is the
                        // airdodge and it is spent the same way.
                        //
                        // The tail is spent standing still, which is what keeps
                        // a blink a dodge: the frames after it are the frames a
                        // whiffed one is punished in.
                        let grounded = p.grounded;
                        p.pos = aim::blink_to(p.pos, dir, dual::dodge_travel(grounded), scene);
                        p.vel.x = Fx::ZERO;
                        p.vel.z = Fx::ZERO;
                        if grounded {
                            Action::Dodge {
                                left: t::dodge_frames(),
                            }
                        } else {
                            p.air_dodged = true;
                            p.vel.y = Fx::ZERO;
                            Action::Dodge {
                                left: t::air_dodge_frames(),
                            }
                        }
                    } else if p.grounded && blinks(p) {
                        // **The Blood mage's dodge as a blink**, behind
                        // `tuning::dodge_blinks`. The frames, the
                        // invulnerability and the commitment are the ordinary
                        // dodge's; the only thing that changes is that she
                        // covers `blink_range` instead of whatever
                        // `dodge_speed` decaying happens to reach, and covers it
                        // in `blink_frames`.
                        //
                        // **Flat, and it stops at the wall.** What this class is
                        // short of is ground rather than height, and the range
                        // is cut back to whatever the arena allows along the
                        // line -- a dodge that went through geometry would be a
                        // different and much worse ability, and the one honest
                        // thing about this answer is that it is only a distance.
                        begin_blink(p, who, dir, scene);
                        Action::Dodge {
                            left: t::dodge_frames(),
                        }
                    } else if p.grounded {
                        p.vel.x = dir.x.mul(t::dodge_speed().mul(foot.dodge));
                        p.vel.z = dir.z.mul(t::dodge_speed().mul(foot.dodge));
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
        .map(|kind| match p.action {
            // The Blood mage holding a charge crawls, whatever the row says
            // about the cast it is the front of.
            Action::Channel { .. }
                if p.class == Class::BloodMage
                    && moves::charge(p.class, kind) == Some(moves::Charge::Tier) =>
            {
                t::charge_crawl() as u8
            }
            _ if holding(p) => t::hold_mobility(),
            _ => moves::get(p.class, kind).mobility,
        })
        .filter(|m| *m > 0)
        .map(|m| t::move_speed().mul(Fx::ratio(m as i32, 100)));
    let steering = ax != 0 || az != 0;

    // **A Grasp reeling her in**, or a blink, which is the same machinery
    // pointed along the stick. Stepped rather than read, because arriving is
    // something it *does*: she lands on the point rather than near it.
    // The Flurry dash's run, as it stood before this frame's step: the flat
    // of it, at its speed, which a jump banked for the arrival carries.
    let flurry_run = (p.class == Class::DualMage
        && p.haul > 0
        && matches!(
            p.action,
            Action::Recovery {
                kind: moves::dual::FLURRY_DASH,
                ..
            }
        ))
    .then(|| {
        let gap = p.haul_to.sub(p.pos);
        V3::new(gap.x, Fx::ZERO, gap.z)
            .normalized()
            .scale(p.haul_speed)
    });
    let hauling = step_haul(p);
    let flurry_arrived = flurry_run.filter(|_| !hauling && p.haul == 0);

    let dashing = shadow::dash_drive(p);
    // Gallows' drive up (2026-10-18). Its clock runs here, once a frame, and
    // on the frame it ends her speed is cut to what she keeps of it.
    let rising = shadow::rise_drive(p);
    if hauling {
        // Ahead of the dash and of everything below it, and it ignores the
        // stick outright: the arms have hold of a wall and she is on the end of
        // them. Like the dash it drives all three axes, so a Grasp thrown up on
        // to a platform's lip takes her up on to it rather than into the side
        // of it -- and like the dash it turns gravity and the arena off for the
        // duration, further down, so the line she travels is the line the arms
        // went out along.
        // The velocity is already under her; `step_haul` set it.
    } else if let Some(drive) = dashing {
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
    } else if matches!(p.action, Action::Dodge { .. }) {
        p.vel.x = p.vel.x.mul(t::dodge_decay());
        p.vel.z = p.vel.z.mul(t::dodge_decay());
    } else if p.action.stunned() {
        // Knockback decays rather than stopping dead, in the air as on the
        // ground. Checked before the airborne branch so a hit connecting
        // mid-jump is not handed the ordinary air control below.
        p.vel.x = p.vel.x.mul(t::stun_decay());
        p.vel.z = p.vel.z.mul(t::stun_decay());
        // **Launched, a fighter still steers** -- the game's answer to
        // Smash's directional influence, and the air control the player
        // already has rather than a second mechanic beside it. The Quake
        // projection turns a velocity without adding to it, so a launched
        // fighter picks *where* they land and never how far. Grounded stun
        // keeps none of it: skidding is not a moment anyone is flying
        // through. And like the rest of the stun rules it is between fighters
        // (`taken` counts only their blows): a creature's throw lands where
        // the creature threw you. See `docs/design/stun.md`.
        if steering && !p.grounded && p.taken > 0 && matches!(p.action, Action::HitStun { .. }) {
            steer_while_stunned(p, move_dir(input.aim_turns(), ax, az), mob.air_speed);
        }
    } else if let Some(drive) = rising {
        // **Gallows** holds its line, all three axes, with gravity off below
        // and the arena on: a ceiling or a wall stops the rise.
        p.vel = drive;
    } else if let Some(drive) = ramming(p) {
        // **The Battering ram** holds its line for its active frames, on the
        // floor and off it: he goes along the ground, not up, and whoever the
        // shield meets goes with him (2026-10-10).
        p.vel.x = drive.x;
        p.vel.z = drive.z;
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
        let speed = if p.swimming() {
            t::move_speed().mul(t::swim_speed())
        } else if p.crouching {
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
    steer_in_the_updraft(p, who, input, scene);

    // **Sliding down a wall** (`cling_to_the_wall`): against one last frame,
    // not rising, and nobody else's -- a fighter knocked into a wall or
    // spiked past one is being thrown, and a throw is not slowed by the wall
    // it goes past. Held to it, unless steering away from it.
    let sliding = keep_sliding(p, on_wall, input);
    if sliding {
        hold_to_the_wall(p, input);
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
    // The second exception to "space in the air does nothing", after the
    // Champion's uppercut below, and it is per class: the Dual mage's **second
    // jump**, once per airtime while her lower bar holds the second tier, and
    // on every press while she is ascending -- the wing beat. A fresh takeoff
    // rather than an addition, so it works on the way down, which is when a
    // second jump is worth having; the sustain is not offered on it, because
    // the height is the tier's to give and not the button's.
    //
    // **Before the ordinary takeoff**, which clears `grounded` on the frame
    // it fires: read after it, the press that left the floor would look like a
    // press in the air and spend the second jump on the first.
    // **The wall jump**, ahead of the second jump for the same reason the
    // second jump is ahead of the takeoff: one press, one jump. Against a
    // wall, or a few frames off one, a press is a jump off it rather than
    // the class's own thing in the air. See `cling_to_the_wall`.
    //
    // **A press kept for the wall** (`tuning::wall_jump_buffer`, 2026-10-20):
    // a press in the air that did nothing else waits that many frames for a
    // wall, and is a wall jump the first frame one is under the hand. With
    // the grace after leaving one, the press has a window round the touch
    // both ways, rather than the frame of it.
    let kept = p.wall_press > 0 && on_wall;
    p.wall_press = p.wall_press.saturating_sub(1);
    let wall_jump = (pressed_space && near_wall || kept)
        && !p.grounded
        && !carrying
        && p.action.actionable()
        && may_wall_jump(p);
    if wall_jump {
        jump_off_the_wall(p, mob.jump, input);
        p.wall_press = 0;
    }
    let mut beat = false;
    if pressed_space
        && !wall_jump
        && !p.grounded
        && p.action.actionable()
        && dual::may_beat_wings(p)
    {
        p.vel.y = t::jump_speed().mul(mob.jump).mul(t::second_jump());
        p.jump_hold = 0;
        dual::spend_wing_beat(p);
        beat = true;
    }
    if pressed_space && !wall_jump && !beat && !p.grounded {
        p.wall_press = t::wall_jump_buffer();
    }

    // Not on the frame a banked dash jump fires: the dash jump below is that
    // press, and both would stack two takeoffs into one.
    let dash_jump = shadow::dash_jump_banked(p) && shadow::carrying_a_dash(p);
    if input.has(Input::SPACE) && p.grounded && p.action.actionable() && !dash_jump {
        p.vel.y = p.vel.y.add(t::jump_speed().mul(mob.jump).mul(foot.jump));
        p.grounded = false;
        p.jump_hold = t::jump_hold_frames();
    }

    // **The dash jump.** Pressed in the dash's last few frames
    // (`bank_dash_jump`, `tuning::dash_jump_buffer`) and thrown on the frame
    // she lands on the shadow, with the crossing's whole speed
    // (`tuning::dash_jump_keep`). Only a press made *before* the halt counts
    // (2026-10-04, from play): a press after it found her already stopped and
    // launched her a few frames late, which read as a stall.
    if dash_jump {
        let lunge = shadow::lunge(p);
        p.vel = V3::new(lunge.x, p.vel.y, lunge.z);
        // And a little more than a jump: the hop of the forward flip she
        // throws out of it (2026-10-10). The flip is drawn off
        // `Shadow::flip`.
        // Both cut to `tuning::dash_jump_rise` since 2026-10-18, with the
        // carry (`tuning::dash_jump_keep`).
        p.vel.y = p.vel.y.add(
            t::jump_speed()
                .mul(mob.jump)
                .add(t::dash_jump_hop())
                .mul(t::dash_jump_rise()),
        );
        shadow::begin_flip(p);
        p.grounded = false;
        p.jump_hold = t::jump_hold_frames();
        p.action = Action::Free;
        shadow::spend_carry(p);
    }

    // **The jump out of the Flurry dash** (2026-10-13): the Reaver's dash jump
    // on hers. Pressed in the run's last `flurry dash, a jump this near the
    // end` frames (`bank_the_flurry_jump`) and thrown the frame she breaks the
    // disk, carrying `flurry dash, the jump keeps` of the run's speed -- the
    // flurry as a way across the valley, not only a way into a fight.
    if let Some(run) = flurry_arrived {
        if dual::take_dash_jump(p) {
            let keep = run.scale(t::flurry_jump_keep());
            p.vel = V3::new(keep.x, t::jump_speed().mul(mob.jump), keep.z);
            p.grounded = false;
            p.jump_hold = t::jump_hold_frames();
            p.action = Action::Free;
        }
    }

    // The second half of the uppercut. You are both off the ground and you
    // have hold of them; pressing jump again takes the pair of you higher,
    // once. This is the "we are settling this in the air" button, and it is the
    // only thing space does while airborne.
    if pressed_space && carrying && !p.leap_used && !p.grounded {
        p.vel.y = p.vel.y.add(t::uppercut_leap());
        p.leap_used = true;
    }

    // Pinned by the Nail: held in the air, and nothing pulls it down.
    let nailed = matches!(p.action, Action::Held { .. }) && p.held_by == NAILED;
    if nailed {
        p.vel = V3::ZERO;
    }
    if !p.grounded && dashing.is_none() && rising.is_none() && !hauling && !nailed {
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
            //
            // **The Reaver's two go where she is looking** (2026-10-12): down
            // the crosshair's line, held inside a cone round straight down
            // (`aim::plunge_dir`), at their own speed -- the whole velocity,
            // so the line she picked is the line she flies.
            if p.class == Class::ShadowReaver {
                p.vel = aim::plunge_dir(p.aim_dir(), t::plunge_cone()).scale(t::plunge_dive());
            } else if p.class == Class::DualMage {
                // **Descent** (2026-10-12): the Plunge's cone and its rule --
                // the line she picked is the line she flies -- at its own
                // speed. A pillar of light coming down where she pointed.
                p.vel = aim::plunge_dir(p.aim_dir(), t::plunge_cone()).scale(t::descent_dive());
            } else {
                p.vel.y = t::landfall_dive().neg();
            }
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
            if sliding {
                gravity = gravity.mul(t::wall_slide_gravity());
            }
            p.vel.y = p.vel.y.add(gravity.mul(DT));
            // Slower at the Dual mage's second tier; one for everybody else.
            let mut floor = t::fall_cap().mul(mob.fall_cap).mul(dual::fall_scale(p));
            if sliding {
                floor = floor.mul(t::wall_slide_fall());
            }
            // **Sail** (2026-10-10): the Bulwark's guard held in the air is a
            // sail, and he falls no faster than it lets him.
            if p.class == Class::Bulwark && p.action.guarding() {
                floor = floor.max(Fx::ZERO.sub(t::sail_fall()));
            }
            if p.vel.y.raw() < floor.raw() {
                // **The wall's grab** (`tuning::wall_slide_catch`): a fall
                // faster than the slide is brought down to it over a few
                // frames, not in one.
                p.vel.y = if sliding {
                    let over = p.vel.y.sub(floor);
                    floor.add(over.mul(Fx::ONE.sub(t::wall_slide_catch())))
                } else {
                    floor
                };
            }
        }
    }

    // **The valley's two props of the air** (`crate::valley`), for anybody:
    // an updraft and falling water. Neither is a class's.
    if dashing.is_none() && !hauling {
        ride_the_air(p, scene.arena);
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
    // The Grasp haul is the same argument: the arms chose a point on a surface
    // (`haul_to_the_grasp` stops her a body's width short of it), so resolving
    // her against the geometry halfway along would catch her feet on the thing
    // she is being pulled on to.
    if dashing.is_some() || hauling {
        return;
    }

    // How fast this fighter was going when the floor arrived. Read before the
    // resolve, which is what stops it.
    let impact = p.vel.y;
    let was_grounded = p.grounded;
    let r = scene.arena.resolve(p.pos, p.vel, was_grounded);
    let wall_normal = r.wall_normal;
    let r = stones::resolve_body(field, r.pos, r.vel, r.grounded, was_grounded);
    p.pos = r.pos;
    p.vel = r.vel;
    p.grounded = r.grounded;
    cling_to_the_wall(p, wall_normal, scene.arena);
    // Every creature there is, in slot order, until one of them has you.
    for (slot, beast) in herd.iter().enumerate() {
        if let Some(beast) = beast {
            meet_the_creature(p, slot, beast);
            if p.aboard() {
                break;
            }
        }
    }
    // And any back a critter offers (`CritterKind::mountable`).
    if !p.aboard() {
        meet_the_critters(p, scene.critters);
    }
    if !p.aboard() {
        catch_a_ledge(p, input, scene.arena);
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
        //
        // The Bulwark's airborne Slam arrives the same way. Only one thrown
        // in the air: a Slam started on the floor is on its own clock, and
        // `was_grounded` is what tells the two apart.
        //
        // **And only once its wind-up is spent.** A Slam pressed a frame
        // before landing is a Slam with fourteen frames still to go, and
        // arriving must not skip them -- otherwise the answer to "how do I
        // make my slowest move instant" is "hop first".
        if let Action::Startup { kind, left } = p.action {
            let slam_from_the_air = !was_grounded && p.class == Class::Bulwark && left == 0;
            if lands_on_the_floor(p.class, kind)
                && (matches!(
                    p.class,
                    Class::Elementalist | Class::ShadowReaver | Class::Champion
                ) || slam_from_the_air)
            {
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
        p.air_stalls = 0;
        p.carpets = 0;
        p.leap_used = false;
        dual::feet_down(p);
    }
}

/// **The wall slide and the wall jump** (2026-10-16): what a body against the
/// side of a solid in the air gets from it, and the record of which wall.
///
/// ```text
///   against a wall, falling   the slide, unless the stick is away: gravity
///                             and terminal velocity much lower
///                             (`wall_slide_gravity`, `wall_slide_fall`), a
///                             faster fall grabbed down to it
///                             (`wall_slide_catch`), held to the wall at
///                             `wall_stick`, the run along it kept
///                             (`wall_slide_keep`), for as far as the wall
///                             goes. Footing for the fall rule: a fall is
///                             measured from where it let go.
///   space, on one, just off   the wall jump: `wall_jump_rise` of a takeoff
///   or just before one        straight up, `wall_jump_push` away from it,
///                             and `wall_jump_carry` the way the stick points
///                             along it. Off the same wall again only from
///                             `wall_jump_drop` below the last, and each one
///                             in an airtime worth `wall_jump_falloff` of
///                             the one before.
/// ```
///
/// **A wall is reached, not given.** The slide is a partial recovery -- some
/// control back, and the fall's height reset to where you let go, so a wall
/// slid all the way down costs nothing to land from -- and it never stops,
/// so a wall is never a perch. (Until 2026-10-20 it was only a little slower
/// than a fall; the owner found it barely felt, and asked for it slower.) The jump does not
/// give back the airdodge, the jump attack or anything else an airtime
/// spends: it is height and a direction, and nothing more.
///
/// **Why there is no ladder.** Two rules, because one is not enough. Off the
/// same wall again only from `wall_jump_drop` below where the last jump off
/// it left (`Wall::same_as`, a face rather than a box) answers jumping off a
/// wall, steering back with an aerial's hang and jumping off it again: every
/// return leaves from lower down, so the climb runs downhill. It does not
/// answer an inside corner, which is two walls a step apart; the falloff
/// does, because the heights it allows add up to a bounded sum however many
/// walls there are. (Until 2026-10-16 the same wall was refused outright;
/// the owner asked for a drop instead, so a wall can be used again on the
/// way down.)
///
/// Called after the resolve with the wall it reported, if it reported one.
fn cling_to_the_wall(p: &mut Player, normal: V3, ground: &Terrain) {
    if p.grounded || p.aboard() || normal == V3::ZERO {
        return;
    }
    // **Only a wall is a wall** (2026-10-20): the side of a thin slab or a
    // post stops the body, but is not slid on or jumped off.
    let face = ground.wall_face(
        p.pos,
        normal,
        t::body_radius(),
        t::body_height(),
        t::wall_face_wide().mul(Fx::ratio(1, 2)),
    );
    if face.raw() < t::wall_face_high().raw() {
        return;
    }
    p.wall = Wall { at: p.pos, normal };
    p.wall_since = 0;
    if p.sliding && slides(p) {
        p.fall_over = p.pos.y.sub(t::fall_free()).max(ground.lowest());
    }
}

/// Is this body one a wall slows: not rising, and not being thrown -- in a
/// stun, or spiked toward the floor.
fn slides(p: &Player) -> bool {
    p.vel.y.raw() <= 0 && !p.action.stunned() && p.slam.raw() <= 0
}

/// **On the slide or off it, by the stick** (2026-10-16): a body falling
/// against a wall is on a slide unless the stick pushes away from it, and
/// pushing away ends one. Since 2026-10-20 (the owner: "easier to get into a
/// slide") the stick along the wall, or let go, starts one too, where until
/// then only into it did: touching a wall is enough, and the only way to
/// fall past one is to mean to. Neither change comes faster than
/// `tuning::wall_slide_flip` after the last (see there for why). A wall gone,
/// a rise, a throw or the floor ends a slide outright, with no debounce: that
/// is the world deciding, not the stick.
fn keep_sliding(p: &mut Player, on_wall: bool, input: Input) -> bool {
    p.slide_flip = p.slide_flip.saturating_sub(1);
    if p.grounded || !on_wall || !slides(p) {
        p.sliding = false;
        return false;
    }
    if p.slide_flip == 0 {
        let want = lean_against(input, p.wall.normal) != Lean::Away;
        if want != p.sliding {
            p.sliding = want;
            p.slide_flip = t::wall_slide_flip();
        }
    }
    p.sliding
}

/// Hold a sliding body to its wall: its speed into the wall becomes
/// `tuning::wall_stick`, and **what it carried along the wall is kept**
/// (2026-10-20), losing `tuning::wall_slide_keep`'s share a frame to
/// friction. The stick along the wall steers it toward
/// `tuning::wall_slide_steer` that way, arrived at by `wall_slide_grip` a
/// frame -- but only ever up to it: speed already going the stick's way
/// faster is the run, and the stick does not brake it. Into the wall, let go,
/// or away from it until the slide ends (`keep_sliding`), only holds it.
///
/// Until 2026-10-20 the slide bled the run down to the steer within a few
/// frames, so a jump off a wall run into at an angle went straight out of it.
fn hold_to_the_wall(p: &mut Player, input: Input) {
    let n = p.wall.normal;
    let across = V3::new(n.z.neg(), Fx::ZERO, n.x);
    let mut along = p
        .vel
        .x
        .mul(across.x)
        .add(p.vel.z.mul(across.z))
        .mul(t::wall_slide_keep());
    if let Lean::Along(dir) = lean_against(input, n) {
        let want = dir.dot(across).mul(t::wall_slide_steer());
        let beyond = if want.raw() >= 0 {
            along.raw() >= want.raw()
        } else {
            along.raw() <= want.raw()
        };
        if !beyond {
            along = along.add(want.sub(along).mul(t::wall_slide_grip()));
        }
    }
    let into = t::wall_stick().neg();
    p.vel.x = across.x.mul(along).add(n.x.mul(into));
    p.vel.z = across.z.mul(along).add(n.z.mul(into));
}

/// **Catching a ledge** (2026-10-16): the forgiving way up. One rule, asked
/// every frame in the air, whether the body walked off the edge, jumped short
/// of it or fell past it -- the owner's, word for word in its parts:
///
/// ```text
///   fully off the edge     no part of the body over the top: its middle at
///                          least its radius out from the rim, and no more
///                          than `ledge_reach` past that
///   not leaving fast       moving away from the face no faster than
///                          `ledge_away` (toward it, any speed)
///   at the hands           the top between `ledge_reach_low` and
///                          `ledge_reach_high` over the feet: about the
///                          shoulder to the reach of the arms overhead
///   not pushing away       the stick into the wall, along it or let go;
///                          away from it is a body that means to go
/// ```
///
/// And only for a body free to act and not rising -- an edge passed on the
/// way up is one a jump is clearing -- and not within `tuning::ledge_regrab`
/// of letting go of one. Only an edge [`Terrain::ledge_near`] says a body fits
/// beside, hanging; whether it can be climbed is asked when the climb is.
fn catch_a_ledge(p: &mut Player, input: Input, ground: &Terrain) {
    if p.ledge.is_some()
        || p.grounded
        || p.aboard()
        || p.ledge_regrab > 0
        || !p.action.actionable()
        || p.health <= 0
        || p.vel.y.raw() > 0
        || p.slam.raw() > 0
    {
        return;
    }
    let Some(edge) = ground
        .ledge_near(
            p.pos,
            t::ledge_reach_low(),
            t::ledge_reach_high(),
            t::ledge_reach(),
            t::ledge_depth(),
        )
        .filter(|e| lean_against(input, e.normal) != Lean::Away)
        .filter(|e| {
            let away = p.vel.x.mul(e.normal.x).add(p.vel.z.mul(e.normal.z));
            away.raw() <= t::ledge_away().raw()
        })
    else {
        return;
    };
    p.ledge = Ledge {
        edge,
        from: p.pos,
        grip: Grip::Catch,
        clock: 0,
        since: 0,
        fresh: false,
    };
    p.vel = V3::ZERO;
    p.sliding = false;
    p.jump_hold = 0;
    p.air_stall = 0;
}

/// **Hanging from a ledge, a frame on** (2026-10-16). Gravity is off and the
/// body is held where the edge says (`Edge::hang`), facing the wall; the hang
/// is footing for the fall rule, as a slide is.
///
/// ```text
///   catching        settled into the hang over `ledge_catch` frames; the
///                   stick does nothing yet
///   nothing         hangs
///   into the wall   climbs up: `ledge_climb` frames, unable to act -- slow,
///                   because the ledge is the forgiving way up and a landed
///                   jump the quick one. Held in since the catch, only once
///                   `ledge_hold_on` has run; pressed in afresh, at once (see
///                   `Ledge::fresh`). Only where there is room on the top
///                   (`Terrain::room_on_top`); under a roof, nothing
///   away from it    lets go into a fall, free to act on the next frame --
///                   quick, the other side of the trade
///   crouch          lets go into a wall slide down the face
///   along it        hand over hand along the edge, for as far as it holds a
///                   body
/// ```
///
/// Into, away and along are `lean_against`'s three envelopes: the stick as it
/// would walk you, by the camera, against the face.
fn hang_on(p: &mut Player, input: Input, ground: &Terrain) {
    let n = p.ledge.edge.normal;
    let depth = t::ledge_depth();
    p.vel = V3::ZERO;
    p.grounded = false;
    p.crouching = false;
    p.jump_hold = 0;
    p.air_stall = 0;
    p.facing = V3::new(n.x.neg(), Fx::ZERO, n.z.neg());
    let lean = lean_against(input, n);
    if lean != Lean::Into {
        p.ledge.fresh = true;
    }
    p.ledge.clock = p.ledge.clock.saturating_add(1);
    p.ledge.since = p.ledge.since.saturating_add(1);
    let clock = p.ledge.clock;
    match p.ledge.grip {
        Grip::Catch => {
            let total = t::ledge_catch();
            let hang = p.ledge.edge.hang(depth);
            p.pos = toward(p.ledge.from, hang, clock, total);
            if clock >= total {
                p.ledge.grip = Grip::Hang;
                p.ledge.clock = 0;
            }
        }
        Grip::Hang => {
            if input.has(Input::CROUCH) {
                return let_go_of_the_ledge(p, true, ground);
            }
            match lean {
                // Only where there is a top to stand on, with room: under a
                // roof, into the wall is as nothing.
                Lean::Into
                    if (p.ledge.fresh || p.ledge.since >= t::ledge_hold_on())
                        && ground.room_on_top(p.ledge.edge) =>
                {
                    p.ledge.grip = Grip::Climb;
                    p.ledge.clock = 0;
                }
                Lean::Away => return let_go_of_the_ledge(p, false, ground),
                Lean::Along(dir) => {
                    let probe = p
                        .ledge
                        .edge
                        .hang(depth)
                        .add(dir.scale(t::ledge_shuffle().mul(DT)));
                    // The same top, a hand further along: the edge found
                    // from there, with a little slack for the step and for
                    // going round an outside corner.
                    let skin = crate::arena::SKIN;
                    let slack = skin.add(skin).add(t::ledge_reach());
                    if let Some(next) =
                        ground.ledge_near(probe, depth.sub(skin), depth.add(skin), slack, depth)
                    {
                        p.ledge.edge = next;
                    }
                }
                Lean::Into | Lean::Still => {}
            }
            p.pos = p.ledge.edge.hang(depth);
        }
        Grip::Climb => {
            // Up the face to the top, then over it on to the top.
            let total = t::ledge_climb();
            let rising = (Fx::from_int(total as i32)
                .mul(t::ledge_climb_rise())
                .to_int() as u8)
                .clamp(1, total);
            let hang = p.ledge.edge.hang(depth);
            let lip = V3::new(hang.x, p.ledge.edge.at.y, hang.z);
            let stand = p.ledge.edge.stand();
            p.pos = if clock <= rising {
                toward(hang, lip, clock, rising)
            } else {
                toward(
                    lip,
                    stand,
                    clock - rising,
                    total.saturating_sub(rising).max(1),
                )
            };
            if clock >= total {
                p.pos = stand;
                p.grounded = true;
                p.ledge = Ledge::NONE;
                // The climb is its own landing: no flight to land from, or
                // a long hang would be drawn arriving like a long fall.
                p.air_frames = 0;
            }
        }
    }
    p.fall_over = p.pos.y.sub(t::fall_free()).max(ground.lowest());
}

/// `clock` frames of `total` of the way from `from` to `to`, in a straight
/// line.
fn toward(from: V3, to: V3, clock: u8, total: u8) -> V3 {
    let k = Fx::ratio(clock.min(total) as i32, total.max(1) as i32);
    from.add(to.sub(from).scale(k))
}

/// **Letting go of a ledge**: from the hang, into a fall pushed a little off
/// the face (`tuning::ledge_let_go`), or -- `into_a_slide` -- against the face
/// on a wall slide. Free to act from the next frame either way, and no ledge
/// caught again for `tuning::ledge_regrab`, nor the slide changed for
/// `wall_slide_flip`.
fn let_go_of_the_ledge(p: &mut Player, into_a_slide: bool, ground: &Terrain) {
    let edge = p.ledge.edge;
    let n = edge.normal;
    let hang = edge.hang(t::ledge_depth());
    p.ledge = Ledge::NONE;
    p.ledge_regrab = t::ledge_regrab();
    p.slide_flip = t::wall_slide_flip();
    // The fall starts here: the hang was not flight, and a landing drawn
    // from the length of the hang would be a heavy one after a short drop.
    p.air_frames = 0;
    p.fall_over = hang.y.sub(t::fall_free()).max(ground.lowest());
    if into_a_slide {
        // Against the face, so the hold into it meets it next frame.
        let r = t::body_radius();
        p.pos = V3::new(edge.at.x.add(n.x.mul(r)), hang.y, edge.at.z.add(n.z.mul(r)));
        p.vel = V3::ZERO;
        p.sliding = true;
        p.wall = Wall {
            at: p.pos,
            normal: n,
        };
        p.wall_since = 0;
    } else {
        p.pos = hang;
        p.vel = n.scale(t::ledge_let_go());
        p.sliding = false;
        // A drop, not a kick off the wall: no wall jump's grace.
        p.wall_since = u8::MAX;
    }
}

/// May this fighter jump off the wall it was last against: any wall but the
/// one the last wall jump left, and that one from `tuning::wall_jump_drop`
/// below where it jumped.
fn may_wall_jump(p: &Player) -> bool {
    if !p.wall.is_some() {
        return false;
    }
    if !p.wall.same_as(p.wall_jumped) {
        return true;
    }
    let lowest = p.wall_jumped.at.y.sub(t::wall_jump_drop());
    p.pos.y.raw() <= lowest.raw()
}

/// **Jump off the wall**: a fresh rise, never less than the body already
/// has, and a push away from the face, keeping the speed along it. `jump` is
/// the class's takeoff multiplier.
///
/// **And the way the stick points along the wall** (2026-10-20): the body is
/// carried that way at least `tuning::wall_jump_carry`, scaled by how much of
/// the stick is along the face -- so a jump off a wall goes out *and
/// forward*, where the player is steering, and a run carried on to the wall
/// faster than that keeps its own speed. The stick straight away from the
/// wall, into it or let go carries nothing more than the run.
fn jump_off_the_wall(p: &mut Player, jump: Fx, input: Input) {
    let mut rise = t::jump_speed().mul(jump).mul(t::wall_jump_rise());
    for _ in 0..p.wall_jumps {
        rise = rise.mul(t::wall_jump_falloff());
    }
    p.vel.y = p.vel.y.max(rise);
    let n = p.wall.normal;
    let away = p.vel.x.mul(n.x).add(p.vel.z.mul(n.z));
    let off = t::wall_jump_push().sub(away.min(Fx::ZERO));
    p.vel.x = p.vel.x.add(n.x.mul(off));
    p.vel.z = p.vel.z.add(n.z.mul(off));
    let (ax, az) = input.move_axis();
    if ax != 0 || az != 0 {
        let stick = move_dir(input.aim_turns(), ax, az);
        let out = stick.x.mul(n.x).add(stick.z.mul(n.z));
        // The stick less its part through the face: along it, as long as
        // the sine of its angle from the normal.
        let side = V3::new(
            stick.x.sub(n.x.mul(out)),
            Fx::ZERO,
            stick.z.sub(n.z.mul(out)),
        );
        let share = side.flat_len();
        if share.raw() > 0 {
            let way = side.normalized();
            let carried = p.vel.x.mul(way.x).add(p.vel.z.mul(way.z));
            let least = t::wall_jump_carry().mul(share);
            if carried.raw() < least.raw() {
                let more = least.sub(carried);
                p.vel.x = p.vel.x.add(way.x.mul(more));
                p.vel.z = p.vel.z.add(way.z.mul(more));
            }
        }
    }
    p.jump_hold = 0;
    p.wall_jumped = Wall {
        at: p.pos,
        normal: p.wall.normal,
    };
    p.wall_jumps = p.wall_jumps.saturating_add(1);
    p.wall_since = u8::MAX;
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
        Class::BloodMage => blood_move(p, input),
        // The Reaver breaks it a third way: right click sends the shadow. It is
        // the one thing in her kit the **crosshair aims**, and the mouse is
        // where aiming lives -- so the mechanic is on the mouse and the swing
        // it displaced went to `E`, which is the one key that does not care
        // where anything is pointed. See `moves::on_e`.
        // The remembered press rather than the button itself: hers is the one
        // mechanic on a click, so it needs an edge of its own, and it is the
        // one input that outlives the frame it was pressed on. See
        // `crate::shadow` for both arguments.
        Class::ShadowReaver => reaver_move(p, input),
        // The Elementalist breaks it a fourth way, and then a fifth. Right
        // click is Cataclysm, for the reason the Reaver's is the mechanic --
        // no shield, so the button is otherwise dead, and Cataclysm is aimed
        // along the crosshair like the auto. And both clicks mean something
        // different once her feet leave the floor, which is the *row* rather
        // than a fifth exception: see `elementalist_move`.
        Class::Elementalist => elementalist_move(
            p,
            Input {
                bits: input.bits & !p.spent_clicks,
                ..input
            },
        ),
        // **Shift is only ever a dodge now**, so on the five classes that kept
        // the shared grammar a click is a poke and nothing modifies it. That
        // strands `SLOT_COMMITTED` on the three that have one and nowhere else
        // to put it -- the Bulwark's Slam, the Elementalist's Fissure, the
        // Blood mage's Rend -- and that is deliberate and temporary: finding
        // each of them a home is a separate job, one kit at a time, and the
        // Reaver's is already on `E`. See `docs/design/controls.md`.
        //
        // **The Bulwark's has one**, since 2026-09-23: the free third click.
        // Slam is where his weight is spent, so it wanted a button of its own
        // rather than a modifier -- see `bulwark-v2.md`. Left first, so both at
        // once is the poke, the same tie-break every other class makes.
        //
        // And on three clicks since 2026-10-10, the last of the six: see
        // `bulwark_move`. No class is on the shared grammar any more.
        Class::Bulwark => bulwark_move(p, input),
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
/// Which move the Blood mage's three clicks throw: left is thorns, middle
/// the spikes, right the scythe -- the row is where her feet are, and a
/// charge in her vessel takes its own click back. See `moves::blood`.
fn blood_move(p: &Player, input: Input) -> Option<u8> {
    use moves::blood as b;
    use moves::blood::keys;
    // Left before middle before right, the tie-break every class makes.
    let click = if input.has(keys::THORNS) {
        keys::THORNS
    } else if input.has(keys::SPIKES) {
        keys::SPIKES
    } else if input.has(keys::SCYTHE) {
        keys::SCYTHE
    } else {
        return None;
    };
    // **A stored charge answers its own click first**, wherever her feet
    // are: the vessel is what a jump or a dodge out of a hold kept, and the
    // click that charged it is the click that lets it go.
    if p.vessel != NO_VESSEL && b::click_of(p.vessel) == click {
        return p.vessel_ready.then_some(p.vessel);
    }
    let takeoff = match click {
        keys::THORNS => b::THORNFALL,
        keys::SPIKES => b::UPHEAVAL,
        _ => b::RISING_REAP,
    };
    // **A tap is the quick thing and a hold is the bigger one** (2026-10-13,
    // 2026-10-18): on the floor and in the air the left click is the Thorn
    // lash and the right click the Sweep, each turning into its hold -- the
    // Bramble, the Reap -- when held past its startup (`countdown`); the
    // middle click is the spike, charged in tiers on the floor and the Stake
    // in the air. Space
    // with a click is a takeoff from the floor, on the window the
    // Elementalist's use (`rise_window`), and **once a jump in the air** as
    // well, as the Reaver's are.
    if p.grounded {
        if !p.aboard() && p.rise.window > 0 {
            return Some(takeoff);
        }
        return Some(match click {
            keys::THORNS => b::THORN_LASH,
            keys::SPIKES => b::BLACK_SPIKE,
            _ => b::SWEEP,
        });
    }
    if !p.aboard() && p.rise.window > 0 && p.rise.lifted <= t::floor_grace() {
        return Some(takeoff);
    }
    if !p.aboard() && input.has(Input::SPACE) && !p.rise.used {
        return Some(takeoff);
    }
    Some(match click {
        keys::THORNS => b::THORN_LASH,
        keys::SPIKES => b::STAKE,
        _ => b::SWEEP,
    })
}

fn elementalist_move(p: &Player, input: Input) -> Option<u8> {
    use moves::elementalist as e;
    use moves::elementalist::keys;
    // Left before middle before right, so a hand that presses two gets an
    // attack rather than silence -- the tie-break every class makes.
    let click = if input.has(keys::EARTH) {
        keys::EARTH
    } else if input.has(keys::FIRE) {
        keys::FIRE
    } else if input.has(keys::WIND) {
        keys::WIND
    } else {
        return None;
    };
    let takeoff = match click {
        keys::EARTH => e::EARTH_JUMP,
        keys::FIRE => e::FIRE_FOUNTAIN,
        _ => e::UPDRAFT,
    };
    // **The row is where her feet are, and space is a row of its own.** On
    // the floor with the jump button's window open, a click leaves the floor
    // -- the Champion's rule, kept in [`Player::rise`] for a class whose
    // mechanic does not keep it. See [`rise_window`].
    if p.grounded {
        if !p.aboard() && p.rise.window > 0 {
            return Some(takeoff);
        }
        return match click {
            // Raise is an instant, not a move: the press edge is read where
            // the mechanic's instants are, beside this. See
            // `raise_on_the_click`.
            keys::EARTH => None,
            keys::FIRE => Some(SLOT_SPECIAL),
            _ => Some(e::AIR_BALL),
        };
    }
    // The first few frames of a jump are still the floor's, as the
    // Champion's are: a click in them is the takeoff from the ground, and
    // `back_to_the_floor` puts her down for it.
    if !p.aboard() && p.rise.window > 0 && p.rise.lifted <= t::floor_grace() {
        return Some(takeoff);
    }
    // **In the air, only the wind has a takeoff.** Space held with right click
    // is the Updraft, once per trip off the ground: it is what stretches a
    // jump. Earth and fire take off from the floor only -- the earth jump is
    // a jump, and the fountain is a burst at her feet.
    if click == keys::WIND && input.has(Input::SPACE) && !p.rise.used && !p.aboard() {
        return Some(e::UPDRAFT);
    }
    Some(match click {
        keys::EARTH => e::LANDFALL,
        keys::FIRE => e::FIRE_CARPET,
        _ => e::GALE,
    })
}

/// Is this one of a class's three takeoffs, on the window kept in
/// [`Player::rise`]? The Elementalist's and the Blood mage's.
fn elementalist_takeoff(class: Class, kind: u8) -> bool {
    use moves::blood as b;
    use moves::elementalist as e;
    match class {
        Class::Elementalist => matches!(kind, e::EARTH_JUMP | e::FIRE_FOUNTAIN | e::UPDRAFT),
        Class::BloodMage => matches!(kind, b::THORNFALL | b::UPHEAVAL | b::RISING_REAP),
        Class::DualMage => moves::dual::is_takeoff(kind),
        Class::ShadowReaver => moves::reaver::is_takeoff(kind),
        Class::Bulwark => moves::bulwark::is_takeoff(kind),
        _ => false,
    }
}

/// Which move `Q` throws: the class special, except on the two classes that
/// put something else there. The Champion has none -- its three clicks are
/// its weapons -- and the Elementalist's is the **weak push**, the Bolt on the
/// floor and the Air bolt off it, since her special went to middle click.
fn keyed_q(p: &Player) -> Option<u8> {
    match p.class {
        Class::Champion => None,
        // The weak push on the floor and off it, like `E`'s strong one: the
        // Air bolt it threw in the air read as something else at the moment a
        // push was wanted (2026-10-09, from play), and is unbound for now.
        Class::Elementalist => Some(SLOT_POKE),
        // Hers is Coagulate, since 2026-10-13.
        Class::BloodMage => Some(moves::blood::COAGULATE),
        // Hers is the dark major, the Abyss; the light one is on `E`.
        Class::DualMage => Some(moves::dual::ABYSS),
        _ => Some(SLOT_SPECIAL),
    }
}

/// **The jump attack's window**, for a class that keeps it on the fighter
/// rather than in its mechanic. Held open while the jump button is down on
/// the floor; counted off the floor; and the trip's takeoff handed back once
/// the feet are down and free. The Champion's three of the same live in
/// `Mechanic::Forms` and are run by `arm_takeoff` and `refresh_the_rise`.
fn rise_window(p: &mut Player, input: Input, field: &Field) {
    if !matches!(
        p.class,
        Class::Elementalist
            | Class::BloodMage
            | Class::DualMage
            | Class::ShadowReaver
            | Class::Bulwark
    ) {
        return;
    }
    let r = &mut p.rise;
    if p.grounded && input.has(Input::SPACE) {
        r.window = t::takeoff_window().min(u8::MAX as u16) as u8;
    } else {
        r.window = r.window.saturating_sub(1);
    }
    r.lifted = if p.grounded {
        0
    } else {
        r.lifted.saturating_add(1)
    };
    if p.grounded {
        r.floor = p.pos.y;
        r.stone = stones::under_in(field, p.pos).map_or(0, |i| i as u8 + 1);
    }
    if p.grounded && p.action.actionable() {
        r.used = false;
    }
}

/// **A takeoff clicked just after the jump comes out of the floor**, put back
/// on the floor it left. The Champion's `rise_from_the_floor`, for the fighter
/// whose window is [`Player::rise`].
fn back_to_the_floor(p: &mut Player, kind: u8, scene: &Scene) {
    if !elementalist_takeoff(p.class, kind) || p.grounded {
        return;
    }
    if p.rise.lifted == 0 || p.rise.lifted > t::floor_grace() || p.vel.y.raw() <= 0 {
        return;
    }
    // Where her feet were last: the stone she jumped off, if it was one.
    // The arena's floor is the fallback for a body that never stood here.
    let floor = p.rise.floor.max(scene.arena.ground_under(p.pos));
    if floor.raw() > p.pos.y.raw() {
        return;
    }
    p.pos.y = floor;
    p.vel.y = Fx::ZERO;
    p.grounded = true;
    p.jump_hold = 0;
}

/// Spend this trip's takeoff, if the move is one.
fn spend_the_rise(p: &mut Player, kind: u8) {
    if elementalist_takeoff(p.class, kind) {
        p.rise.used = true;
        p.rise.window = 0;
    }
}

/// **Raise, on the earth click's press.** An instant with no frames, as it
/// was on `E`: a stone at the crosshair, and the click held past its rise is
/// Fissure's charge. Only on the floor, only free, and never while the jump
/// window is open -- then the same click is the earth jump.
fn raise_on_the_click(p: &Player, pressed_earth: bool) -> bool {
    pressed_earth
        && p.class == Class::Elementalist
        && p.grounded
        && !p.aboard()
        && p.rise.window == 0
}

/// Which move `F` throws, here and now.
///
/// Unaimed things live on the key the hand that does not aim can reach, and
/// the Elementalist's are the two drafts: a column of air on her own body,
/// blowing up with her feet on the floor and down with them off it. Nobody
/// else reads the key yet. See `moves::elementalist::UPDRAFT`.
fn keyed_f(p: &Player) -> Option<u8> {
    match p.class {
        // The Updraft went to space and right click on 2026-10-09; the key
        // it left carries the Cinder spray, whose click the pillar took.
        Class::Elementalist if p.grounded => Some(moves::elementalist::CINDER),
        Class::Elementalist => Some(moves::elementalist::DOWNDRAFT),
        // The Dual mage's dark spell, on the key the hand that moves can
        // reach, since her clicks became her autos (2026-10-12).
        Class::DualMage => Some(moves::dual::DEVOUR),
        _ => None,
    }
}

/// Which move `R` throws: Tremor, the Quake on her own feet, with them on
/// the floor. Off it the key carries nothing yet.
fn keyed_r(p: &Player) -> Option<u8> {
    match p.class {
        Class::Elementalist if p.grounded => Some(moves::elementalist::TREMOR),
        // And her light one.
        Class::DualMage => Some(moves::dual::SMITE),
        _ => None,
    }
}

/// Which move the second side button throws: Quake, wherever the crosshair
/// is on the floor, from the ground or the air alike.
fn keyed_side_b(p: &Player) -> Option<u8> {
    match p.class {
        Class::Elementalist => Some(moves::elementalist::QUAKE),
        _ => None,
    }
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
    // **The strong push**, since 2026-10-09: Cataclysm, from the floor and
    // the air alike. Raise went to the earth click and Landfall with it.
    if p.class == Class::Elementalist {
        return Some(SLOT_HEAVY);
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
        && ((p.class == Class::Elementalist && kind == moves::elementalist::LANDFALL)
            || (p.class == Class::ShadowReaver && kind == moves::reaver::PLUNGE)
            || (p.class == Class::DualMage && kind == moves::dual::DESCENT))
}

/// Is this startup one that waits for the floor rather than for its own clock?
///
/// Two do. Landfall, and the Bulwark's **Slam thrown in the air**: he drives
/// the shield into the ground, so the shake is where the ground is and it comes
/// when his feet do -- which is also what lets the fall into it be counted,
/// because the fall is not over until then. Unlike Landfall it is not driven:
/// he falls the way he was falling, and how fast is what it pays.
fn waits_for_the_floor(p: &Player, kind: u8) -> bool {
    !p.grounded && lands_on_the_floor(p.class, kind)
}

/// The moves whose active frames begin on the frame the feet arrive.
fn lands_on_the_floor(class: Class, kind: u8) -> bool {
    (class == Class::Champion
        && matches!(kind, moves::champion::METEOR | moves::champion::SHOCKWAVE))
        || (class == Class::Elementalist && kind == moves::elementalist::LANDFALL)
        || (class == Class::Bulwark && kind == SLOT_COMMITTED)
        || (class == Class::ShadowReaver && kind == moves::reaver::PLUNGE)
        || (class == Class::DualMage && kind == moves::dual::DESCENT)
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
/// Which of the Bulwark's seven a click asks for (2026-10-10): **the strike,
/// the weight, the guard** -- left, middle, right -- on the floor, in the air,
/// and off the floor with space, on the shared window (`Player::rise`). Right
/// click on its own is the guard, which is not a move: it is only a move as
/// the Shield step. Left first, so both at once is the strike, the tie-break
/// every class makes. See
/// `docs/design/exploration/0012_bulwark_on_three_clicks.md`.
fn bulwark_move(p: &Player, input: Input) -> Option<u8> {
    use moves::bulwark as b;
    use moves::bulwark::keys;
    let window =
        !p.aboard() && p.rise.window > 0 && (p.grounded || p.rise.lifted <= t::floor_grace());
    if window {
        return if input.has(keys::STRIKE) {
            Some(b::RAM)
        } else if input.has(keys::WEIGHT) {
            Some(b::UNLOAD)
        } else if input.has(keys::GUARD) && p.shield().is_some_and(|s| s.in_hand()) {
            Some(b::SHIELD_STEP)
        } else {
            None
        };
    }
    if input.has(keys::STRIKE) {
        Some(if p.grounded { b::BASH } else { b::REBOUND })
    } else if input.has(keys::WEIGHT) {
        Some(b::SLAM)
    } else {
        None
    }
}

/// Which of the Reaver's eleven a click asks for (2026-10-10): **finishers,
/// neutral, the shadow** -- left, middle, right -- on the floor, in the air,
/// and off the floor with space, on the shared window (`Player::rise`).
/// Right click is still the remembered press (`shadow::order_queued`), so the
/// mechanic cannot be eaten by her own frames; what it throws is the row's.
/// See `docs/design/exploration/0013_shadow_reaver_dual_blades.md`.
fn reaver_move(p: &Player, input: Input) -> Option<u8> {
    use moves::reaver as r;
    use moves::reaver::keys;
    let window =
        !p.aboard() && p.rise.window > 0 && (p.grounded || p.rise.lifted <= t::floor_grace());
    // Space with a click in the air, once a trip off the ground: Gallows and
    // the Tumble are hers there too (2026-10-12).
    let in_the_air = !p.grounded && !p.aboard() && input.has(Input::SPACE) && !p.rise.used;
    if shadow::order_queued(p) {
        return Some(if window {
            r::RISING_WHIRL
        } else if p.grounded {
            r::SEND
        } else {
            r::WHIRL
        });
    }
    let click = if input.has(keys::FINISHER) {
        keys::FINISHER
    } else if input.has(keys::NEUTRAL) {
        keys::NEUTRAL
    } else {
        return None;
    };
    if window || in_the_air {
        return Some(if click == keys::FINISHER {
            r::GALLOWS
        } else {
            r::TUMBLE
        });
    }
    Some(match (click, p.grounded) {
        (keys::FINISHER, true) => r::REAVE,
        (keys::FINISHER, false) => r::PLUNGE,
        (_, true) => r::SHURIKEN,
        (_, false) => r::FAN,
    })
}

fn dual_move(p: &Player, input: Input) -> Option<u8> {
    use moves::dual as d;
    use moves::dual::keys;
    // Left before middle before right, the tie-break every class makes.
    let click = if input.has(keys::DARK) {
        keys::DARK
    } else if input.has(keys::TWILIGHT) {
        keys::TWILIGHT
    } else if input.has(keys::LIGHT) {
        keys::LIGHT
    } else {
        return None;
    };
    // **The bare clicks are her autos, on the floor and in the air alike**
    // (2026-10-12, the second pass): a wing off that arm and a hop after it.
    // A click that meant one thing standing and another a metre up turned
    // under her every time a flurry lifted her, which is what kept tripping
    // the owner up -- see `docs/design/exploration/0016_dual_mage_controls.md`.
    // Space and a click in a jump's first frames is still a takeoff, from the
    // floor only: in the air space is her second jump and her wing beat.
    let takeoff = match click {
        keys::DARK => d::VEIL,
        keys::TWILIGHT => d::EQUINOX,
        _ => d::SANCTUARY,
    };
    let rising =
        !p.aboard() && p.rise.window > 0 && (p.grounded || p.rise.lifted <= t::floor_grace());
    if rising {
        return Some(takeoff);
    }
    // **Every click opens its long hold, and is a tap until it fills**
    // (2026-10-13, from the owner: "either / or, never both animations even
    // in part"). Let go before it is full and the tap comes out then -- the
    // wing, or Binary or Phase for the middle click (`step_channel`); hold it
    // full and the spell does, and the tap never began. With the spell
    // locked out the tap is thrown at once.
    let (hold, tap) = match click {
        keys::DARK => (d::CHAIN_LIGHTNING, Some(d::DARK_AUTO)),
        keys::LIGHT => (d::DISK_RAY, Some(d::LIGHT_AUTO)),
        _ => (d::FLURRY_DASH, twilight_tap(p)),
    };
    if !p.locked_out(hold) { Some(hold) } else { tap }
}

/// What a tap of a click is, for the hold it opened: the wing for the dark
/// and light holds, Binary or Phase for the Flurry dash's.
fn tap_of(p: &Player, hold: u8) -> Option<u8> {
    use moves::dual as d;
    match hold {
        d::CHAIN_LIGHTNING => Some(d::DARK_AUTO),
        d::DISK_RAY => Some(d::LIGHT_AUTO),
        _ => twilight_tap(p),
    }
}

/// What a tap of the middle click is: Binary on the floor, Phase in the air --
/// which spends the airdodge, so once a trip.
fn twilight_tap(p: &Player) -> Option<u8> {
    use moves::dual as d;
    if p.grounded {
        Some(d::BINARY)
    } else {
        (!p.air_dodged).then_some(d::PHASE)
    }
}

/// **The Flurry dash's dash** (2026-10-12; 2026-10-13, to the disk): on the
/// frame the disk has stood its moment she runs to it -- the spot the flurry
/// was thrown at, stopped short of anything solid on the way
/// (`aim::flurry_dash_to`) -- at `flurry dash, speed`, untouchable for the
/// length of it, and breaks it on arrival (the burst `World` throws). What is
/// left of the recovery is the run and the move's own. A jump timed for the
/// arrival keeps the run (`bank_the_flurry_jump`). A no-op for everybody else.
fn dash_down_the_flurry(p: &mut Player, scene: &Scene) {
    let Action::Active {
        kind: moves::dual::FLURRY_DASH,
        left: 0,
    } = p.action
    else {
        return;
    };
    if p.class != Class::DualMage || p.aboard() {
        return;
    }
    let m = moves::get(p.class, moves::dual::FLURRY_DASH);
    let to = aim::flurry_dash_to(p.pos, p.aim_path.to, scene);
    let far = to.sub(p.pos).len();
    let speed = t::flurry_dash_speed();
    let frames = far.div(speed.mul(DT)).to_int().clamp(1, 120) as u16 + 1;
    begin_haul(p, to, speed, frames + 1);
    dual::veil_for(p, frames.saturating_add(1).min(u8::MAX as u16) as u8);
    p.action = Action::Recovery {
        kind: moves::dual::FLURRY_DASH,
        left: frames.saturating_add(m.recovery),
    };
}

/// A jump pressed while the Flurry dash is still running: kept for the
/// arrival if she is within `flurry dash, a jump this near the end` frames of
/// the disk, and thrown the frame she breaks it. Earlier than that it is
/// dropped -- the Reaver's dash jump, word for word. A no-op for everybody else.
fn bank_the_flurry_jump(p: &mut Player) {
    let running = p.class == Class::DualMage
        && p.haul > 0
        && matches!(
            p.action,
            Action::Recovery {
                kind: moves::dual::FLURRY_DASH,
                ..
            }
        );
    if !running {
        return;
    }
    let reach = p
        .haul_speed
        .mul(DT)
        .mul(Fx::from_int(t::flurry_jump_buffer() as i32));
    if p.haul_to.sub(p.pos).len().raw() <= reach.raw() {
        dual::bank_dash_jump(p);
    }
}

/// **The hop after an auto** (2026-10-12, the second pass): on the frame one
/// of her wings finishes, if she is holding a direction she hops that way --
/// `auto, hop` along the floor and `dual::hop_lift` up, as far as the arena
/// lets her (`aim::hop_to`) -- and what is left of the recovery is the hop and
/// `auto, recovery after a hop`. Holding nothing, she stands and recovers in
/// full. Kalista's step: a flurry of hits is a flurry of hops, and the hops
/// are where her mobility lives. A no-op for everybody else.
fn hop_after_the_auto(p: &mut Player, input: Input, scene: &Scene) {
    let Action::Active { kind, left: 0 } = p.action else {
        return;
    };
    if p.class != Class::DualMage || !moves::dual::is_an_auto(kind) || p.aboard() {
        return;
    }
    let (ax, az) = input.move_axis();
    if ax == 0 && az == 0 {
        return;
    }
    let dir = move_dir(input.aim_turns(), ax, az);
    // **In the air the hop is a jump**, once an airtime, and every time while
    // ascending: it rises `auto, the air hop rises` higher than a hop on the
    // floor would. After that an auto in the air only carries her.
    let mut lift = dual::hop_lift(p);
    if !p.grounded && dual::may_air_hop(p) {
        lift = lift.add(t::auto_air_jump());
        dual::spend_air_hop(p);
    }
    let to = aim::hop_to(p.pos, dir, t::auto_hop(), lift, scene);
    let far = to.sub(p.pos).len();
    let speed = t::auto_hop_speed();
    let frames = far.div(speed.mul(DT)).to_int().clamp(1, 120) as u16 + 1;
    begin_haul(p, to, speed, frames + 1);
    p.action = Action::Recovery {
        kind,
        left: frames.saturating_add(t::auto_hop_recovery()),
    };
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
///
/// **A press, not a button that is down** (2026-10-20, from play): only a
/// weapon pressed and not yet spent asks for anything -- `Fury::fresh`, kept
/// a few frames. Holding the button used to run the string by itself, sword
/// after sword, and the rhythm the three hits are built on was something a
/// held button did for you rather than something you played. The press is
/// read off `Fury`, where `find_the_beat` put it this frame, so one already
/// let go still counts while it is kept.
fn champion_weapon(p: &Player) -> Option<u8> {
    use moves::champion as c;
    let Mechanic::Forms { fury, .. } = p.mechanic else {
        return None;
    };
    let pressed = fury.fresh;
    if pressed & 1 != 0 {
        Some(c::SWORD)
    } else if pressed & 2 != 0 {
        Some(c::HAMMER)
    } else if pressed & 4 != 0 {
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
    let weapon = champion_weapon(p)?;
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
    //
    // **On the floor**, the window: the jump button down makes a click the
    // rising attack from the ground.
    if p.grounded && !p.aboard() && champion_takeoff_window(p) > 0 {
        return Some(c::TAKEOFF + weapon);
    }
    // **In the air**, one rule (2026-10-05, from play): holding jump is the
    // rising attack, once per trip off the ground; otherwise the aerial. It
    // used to be the window running eight frames into the jump, which made a
    // click a frame either side of its end a different move -- the rising
    // attack from a metre up, or the hammer's spike before the apex.
    //
    // And the first few frames of a jump are still the floor's: a click in
    // them is the rising attack from the ground whether or not jump is still
    // held (`rise_from_the_floor` puts him back down for it).
    if !p.grounded {
        let just_left = matches!(
            p.mechanic,
            Mechanic::Forms { lifted, takeoff, .. } if lifted <= t::floor_grace() && takeoff > 0
        );
        if !p.aboard() && !rise_spent(p) && (just_left || input.has(Input::SPACE)) {
            return Some(c::TAKEOFF + weapon);
        }
        // **The third hit from the air** (2026-10-13, from play) is the
        // aerial: with two hits behind him it is the string's third --
        // it ends it, gives the Rush back when it lands, and held it is the
        // grand finisher (`begin_champion` gives it its place). The button
        // is the weapon and the feet pick the move, so the move stays the
        // air's own. Throwing the ground's third links from the air instead
        // was tried: the hunt bots struck a creature's back a quarter less
        // often, 187 times against 258 across 24 hunts.
        return Some(c::IN_THE_AIR + weapon);
    }
    Some(c::link(string_stage(p), weapon))
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

/// How many hits of the string are behind the next one: 0, 1 or 2. Past the
/// third the window runs on into the next string (2026-10-20, from play:
/// keeping the beat is rewarded through the third), and the next hit is
/// that string's first.
fn string_stage(p: &Player) -> u8 {
    match champion_chain(p).0 {
        c if c >= moves::champion::DEPTH => 0,
        c => c,
    }
}

/// Is this trip off the ground's rising attack spent?
fn rise_spent(p: &Player) -> bool {
    matches!(
        p.mechanic,
        Mechanic::Forms {
            rise_used: true,
            ..
        }
    )
}

/// Give the rising attack back once he is on his feet and free: the trip it
/// belonged to is over. Not while a takeoff thrown from the floor is still
/// winding up on it, which is the same trip.
fn refresh_the_rise(p: &mut Player) {
    let (grounded, free) = (p.grounded, p.action.actionable());
    if let Mechanic::Forms {
        rise_used, lifted, ..
    } = &mut p.mechanic
    {
        *lifted = if grounded {
            0
        } else {
            lifted.saturating_add(1)
        };
        if grounded && free {
            *rise_used = false;
        }
    }
}

/// **A rising attack clicked just after the jump comes out of the floor.**
/// In the first `tuning::floor_grace` frames of a trip off the ground he is
/// put back on the floor he left -- under half a metre, hidden by the plant
/// the move starts with -- and the takeoff runs from there, as it would have
/// with the click a frame earlier. Later than that, it is the airborne rising
/// attack, from where he is. Added 2026-10-05, from play: a click a few frames
/// after jump threw it from a metre up, over the heads it was meant for.
fn rise_from_the_floor(p: &mut Player, kind: u8, scene: &Scene) {
    if p.class != Class::Champion || p.grounded || !moves::champion::is_takeoff(kind) {
        return;
    }
    let Mechanic::Forms { lifted, .. } = p.mechanic else {
        return;
    };
    if lifted == 0 || lifted > t::floor_grace() || p.vel.y.raw() <= 0 {
        return;
    }
    // Where her feet were last: the stone she jumped off, if it was one.
    // The arena's floor is the fallback for a body that never stood here.
    let floor = p.rise.floor.max(scene.arena.ground_under(p.pos));
    if floor.raw() > p.pos.y.raw() {
        return;
    }
    p.pos.y = floor;
    p.vel.y = Fx::ZERO;
    p.grounded = true;
    p.jump_hold = 0;
}

/// **Jump just after a click.** A grounded swing still in the first frames of
/// its startup (`tuning::takeoff_late`) becomes the rising attack with the
/// same weapon when jump is pressed: the click-first half of "attack as you
/// jump", which used to throw the grounded swing and then, out of its
/// recovery, a rising attack nobody asked for. The swing's repeat lockout is
/// handed back, since it never came out. Added 2026-10-05, from play.
fn rise_out_of_a_swing(
    p: &mut Player,
    who: usize,
    pressed_space: bool,
    input: Input,
    scene: &Scene,
) {
    use moves::champion as c;
    if p.class != Class::Champion || !pressed_space || !p.grounded || p.aboard() {
        return;
    }
    let Action::Startup { kind, left } = p.action else {
        return;
    };
    if kind >= c::IN_THE_AIR {
        return;
    }
    let m = moves::get(p.class, kind);
    if m.startup.saturating_sub(left) > t::takeoff_late() {
        return;
    }
    if let Some(slot) = p.repeat_lock.get_mut(kind as usize) {
        *slot = 0;
    }
    let rise = c::TAKEOFF + c::weapon(kind);
    begin_champion(p, rise);
    p.action = begin_move(p, who, rise, input, scene, true);
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
fn chain_cancel(p: &mut Player) {
    use moves::champion as c;
    if p.class != Class::Champion {
        return;
    }
    let Action::Recovery { kind, left } = p.action else {
        return;
    };
    if !c::in_string(kind) {
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
    let Some(next) = champion_weapon(p) else {
        return;
    };
    let m = moves::get(p.class, kind);
    if m.recovery.saturating_sub(left) < recovery_owed(kind, next != c::weapon(kind)) {
        return;
    }
    // Free rather than straight into the next link: the ordinary input path is
    // three lines below and already knows how to throw one, including the
    // lockout, the aim and the animation. A second way to start a move is a
    // second way for one of those to be forgotten.
    p.action = Action::Free;
}

/// **The Champion's tempo** (2026-10-20, from play): the window for the next
/// hit of his string and its sweet spot, as the HUD's tempo bar draws it. A
/// read of the state, never a second statement of a rule: the window is the
/// chain's own clock (`tuning::chain_grace`, counted from the moment he may
/// act again) and the sweet spot is the Beat's (`tuning::beat_from`).
///
/// ```text
///   0 ───── beat_from ── beat_to ────────────────── end
///   │ he may act │ the Beat │ the string still lives │ over
/// ```
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Tempo {
    /// The hit the next press throws: 0, 1 or 2 for the first, second and
    /// third -- after the third, the next string's first.
    pub next: u8,
    /// Which hits of the string landed, as bits: the first hit is bit 0.
    pub landed: u8,
    /// A finisher is running: the string is over, there is no window.
    pub finishing: bool,
    /// The window, in frames since he could act again: its sweet spot, its
    /// end, and now. `end` is zero outside a string; `now` is nought while
    /// a link still runs.
    pub beat_from: u16,
    pub beat_to: u16,
    pub end: u16,
    pub now: u16,
}

/// The Champion's [`Tempo`]; `None` for every other class.
pub fn tempo(p: &Player) -> Option<Tempo> {
    let Mechanic::Forms { chain, fury, .. } = p.mechanic else {
        return None;
    };
    let finishing = p.action.attack_kind().is_some_and(moves::champion::is_full);
    if chain == 0 {
        return Some(Tempo {
            finishing,
            ..Tempo::default()
        });
    }
    Some(Tempo {
        next: string_stage(p),
        landed: fury.landed,
        finishing,
        beat_from: t::beat_from(),
        beat_to: t::beat_to(),
        end: t::chain_grace(),
        now: beat_clock(p),
    })
}

/// Has the link running now reached the frame the next one may cut its
/// recovery at? The rule [`chain_cancel`] cuts on, asked of the earliest
/// weapon -- a different one -- and never of the third, which has no next.
fn may_cancel(p: &Player) -> bool {
    use moves::champion as c;
    let Action::Recovery { kind, left } = p.action else {
        return false;
    };
    let Mechanic::Forms {
        chain,
        chain_hit: true,
        ..
    } = p.mechanic
    else {
        return false;
    };
    c::in_string(kind)
        && chain < c::DEPTH
        && moves::get(p.class, kind).recovery.saturating_sub(left) >= recovery_owed(kind, true)
}

/// How much of a connected link's recovery is served before the next link may
/// start: a share of its own, smaller into a `swapped` weapon. The one
/// statement of it -- [`chain_cancel`] cuts the recovery here, and the tempo
/// bar draws its marks here (`tempo`).
fn recovery_owed(kind: u8, swapped: bool) -> u16 {
    let share = if swapped {
        t::chain_cancel_swapped()
    } else {
        t::chain_cancel_repeated()
    };
    (moves::get(Class::Champion, kind).recovery as u32 * share as u32 / 100) as u16
}

/// Cut the **Fire carpet's** recovery short into whatever she throws next.
///
/// The carpet is laid to be used: an Updraft into it is a Thermal, a Gale
/// down it comes out lit. Waiting the whole recovery out left her falling by
/// the time the follow-up came, and the carpet's hang had already taken most
/// of her rise, so the combo it exists for was a thing nobody could reach
/// (2026-10-10, from play).
///
/// **A window, not a licence.** It opens a few frames into the recovery
/// (`tuning::carpet_cancel_from`) and stays open for a few more
/// (`tuning::carpet_cancel_for`); a move asked for in it comes out now, and
/// one asked for after it waits for the rest, as before. Recovery only, for
/// the reason the Reaver's cancel is: the carpet is already out. Only into a
/// move that would actually come out -- a second carpet is still locked out
/// -- so the cancel cannot be spent on nothing.
///
/// `pressed` is the press edges of `E`, `F`, `R` and the second side button,
/// which are read on the press; the clicks and `Q` are read held, as the
/// input below reads them.
fn cancel_the_carpet(
    p: &mut Player,
    input: Input,
    out: &[bool; moves::MAX_SLOTS],
    pressed: [bool; 4],
) {
    if p.class != Class::Elementalist {
        return;
    }
    let Action::Recovery { kind, left } = p.action else {
        return;
    };
    if kind != moves::elementalist::FIRE_CARPET {
        return;
    }
    let into = moves::get(p.class, kind).recovery.saturating_sub(left);
    let from = t::carpet_cancel_from();
    if into < from || into >= from.saturating_add(t::carpet_cancel_for()) {
        return;
    }
    let [mechanic, f, r, side_b] = pressed;
    let asked = [
        clicked_move(p, input),
        input.has(Input::SPECIAL).then(|| keyed_q(p)).flatten(),
        mechanic.then(|| keyed_move(p)).flatten(),
        f.then(|| keyed_f(p)).flatten(),
        r.then(|| keyed_r(p)).flatten(),
        side_b.then(|| keyed_side_b(p)).flatten(),
    ];
    if asked.into_iter().flatten().any(|k| p.can_throw(k, out)) {
        // Free rather than straight into the move, as `chain_cancel` does:
        // the ordinary input path below already knows how to throw it.
        p.action = Action::Free;
    }
}

/// **A jump cuts Fissure short while she stands on a stone** (2026-10-21,
/// from play). Held at her own feet, the stone erupts under her on the move's
/// first active frame and climbs, carrying her -- and the move's last active
/// frames and its recovery kept the jump locked until the climb was long
/// over, so the structure jump the surge was meant to be never came out.
/// From the frame after the surge is sent (`hit_used`, which
/// `World::race_the_crack` sets), space with her feet on a stone ends the
/// move, and the ordinary jump below takes off from the stone with its climb.
/// The surge is already out, so nothing it does is given up.
fn jump_off_the_crack(p: &mut Player, input: Input, scene: &Scene) {
    if p.class != Class::Elementalist || !input.has(Input::SPACE) || !p.grounded {
        return;
    }
    let kind = match p.action {
        Action::Active { kind, .. } | Action::Recovery { kind, .. } => kind,
        _ => return,
    };
    if kind != SLOT_COMMITTED || !p.hit_used {
        return;
    }
    if stones::under_in(scene.stones, p.pos).is_some() {
        p.action = Action::Free;
    }
}

/// Cut the recovery of anything the **Elementalist** throws short into `Q` or
/// `E`, the two pushes.
///
/// The clicks are what she makes and the keys are what she does to it, so
/// nearly every push follows something she has just put down. Waiting out the
/// whole recovery first made every push two moves of frames -- the stone's
/// recovery and then the push's own wind-up -- and the second wind-up is the
/// part the opponent was meant to read (2026-10-21, from play).
///
/// **From anything but a push**, from `tuning::push_cancel_from` frames into
/// its recovery to the end of it. Recovery only, as the carpet's: the thing is
/// already out. Not a push into a push: the Bolt then Cataclysm is two
/// decisions about one stone, and paying for the first is the price of
/// making it. Only into a push that would come out, so the cancel is never
/// spent on nothing. What it cannot buy is safety: she is still in the
/// push's own frames, which are about as long as any recovery they cut
/// (`a_cancelled_recovery_is_still_punishable_on_block`).
///
/// `Q` is read held and `E` on its press (`pressed_e`), as the input below
/// reads them.
fn cancel_into_a_push(
    p: &mut Player,
    input: Input,
    out: &[bool; moves::MAX_SLOTS],
    pressed_e: bool,
) {
    if p.class != Class::Elementalist {
        return;
    }
    let Action::Recovery { kind, left } = p.action else {
        return;
    };
    if kind == SLOT_POKE || kind == SLOT_HEAVY {
        return;
    }
    if moves::get(p.class, kind).recovery.saturating_sub(left) < t::push_cancel_from() {
        return;
    }
    let asked = [
        input.has(Input::SPECIAL).then(|| keyed_q(p)).flatten(),
        pressed_e.then(|| keyed_move(p)).flatten(),
    ];
    if asked.into_iter().flatten().any(|k| p.can_throw(k, out)) {
        // Free, as `cancel_the_carpet` does: the input below throws it, and
        // reads `Q` and `E` before anything else.
        p.action = Action::Free;
    }
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
    let Some(link) = place_in_string(p, kind) else {
        return;
    };
    if let Mechanic::Forms {
        chain_hit,
        recharge,
        rush,
        fury,
        ..
    } = &mut p.mechanic
    {
        *chain_hit = true;
        fury.landed |= 1 << link.min(moves::champion::DEPTH - 1);
        // **The refund** (exploration 0015, C1): a third hit that lands gives
        // the Rush back, so the way out -- or back in -- is already there when
        // the string is over. Not mid-dash: a Rush cannot refill itself.
        if link + 1 == moves::champion::DEPTH && *rush == 0 {
            *recharge = 0;
        }
    }
}

/// Where the chain is after this move: one hit deeper, or back to nothing.
///
/// **The nine links, the aerials and the takeoffs are in it** (2026-10-13,
/// from play: "I find myself counting to three"). A link is its own place; an
/// aerial or a takeoff is the next one, however many hits were behind it, so
/// ground, jump, third is three. A Rush move ends the chain outright: the Rush
/// is its own way in.
fn advance_chain(p: &mut Player, kind: u8) {
    use moves::champion as c;
    let stage = match c::link_of(kind) {
        Some(link) => link + 1,
        None if c::in_string(kind) => (string_stage(p) + 1).min(c::DEPTH),
        None => 0,
    };
    if let Mechanic::Forms {
        chain,
        chain_left,
        chain_hit,
        fury,
        ..
    } = &mut p.mechanic
    {
        *chain = stage;
        *chain_left = if stage > 0 { t::chain_grace() } else { 0 };
        *chain_hit = false;
        // A new string starts with nothing landed.
        if stage <= 1 {
            fury.landed = 0;
        }
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
        // The stab **carries** (exploration 0015, C1): it used to be the one
        // Rush move that stopped dead, and it landed none of seventeen in the
        // owner's tape. It rides the run out now, and what it catches it
        // holds on the point and takes along (its `grabs`): the run is kept
        // alive through the thrust and for as long as the hold, so there is a
        // run left to carry them on.
        c::RUSH_STAB => {
            let m = moves::get(p.class, kind);
            (rush.max(m.startup + m.active + m.grabs), rush_vel)
        }
        // Everything else rides the dash out.
        _ => (rush, rush_vel),
    };
    if kind == c::UPPERCUT {
        p.leap_used = false;
    }
    // A link thrown on the beat is a true link; anything else spends the
    // beat on nothing. See `find_the_beat`.
    // Where in the string this move stands: a link's own place, or for an
    // aerial or a takeoff the hits already behind it.
    let place = c::link_of(kind).unwrap_or(string_stage(p));
    if let Mechanic::Forms { fury, .. } = &mut p.mechanic {
        fury.true_link = c::in_string(kind) && fury.on_beat;
        fury.on_beat = false;
        fury.early = false;
        if c::in_string(kind) {
            fury.place = place;
        }
        // A move may be held only if its button was pressed for it. The press
        // is spent here, so a button held on past this link throws nothing
        // more: the next link wants a press of its own (2026-10-20).
        // Kept a few frames after it went down (`Fury::fresh`), a press may
        // already be let go when its link starts: then it is a tap.
        let bit = 1u8 << c::weapon(kind);
        fury.may_hold = c::holdable(kind) && fury.fresh & fury.clicks & bit != 0;
        fury.fresh &= !bit;
    }
    p.channelled = Fx::ZERO;
    // The chain moves on, or ends, depending on what this move was.
    advance_chain(p, kind);
    if let Mechanic::Forms {
        form,
        rush: r,
        rush_vel: v,
        takeoff,
        liquid,
        ..
    } = &mut p.mechanic
    {
        // The weapon is whichever button was pressed. Nothing else sets it:
        // there is no mode to be in any more.
        let from = *form;
        *form = Form::of_move(kind);
        liquid.call(from);
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
    if c::is_takeoff(kind) {
        if let Mechanic::Forms { rise_used, .. } = &mut p.mechanic {
            *rise_used = true;
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

/// **The Updraft thrown from the floor**: higher than a jump, and a push the
/// way she is steering.
///
/// A takeoff spends the trip's: from the floor she gives up the Updraft she
/// could have had in the air, and the jump attack along with it, which costs
/// her far more than pressing space does. So it buys more than space -- see
/// `tuning::updraft_floor_lift`. The push is along the walk she is holding
/// and nothing without one, so straight up stays straight up.
fn updraft_off_the_floor(p: &mut Player, input: Input) {
    p.vel.y = p.vel.y.max(t::updraft_floor_lift());
    let (ax, az) = input.move_axis();
    if ax == 0 && az == 0 {
        return;
    }
    let dir = move_dir(input.aim_turns(), ax, az);
    let push = t::updraft_floor_push();
    p.vel.x = p.vel.x.add(dir.x.mul(push));
    p.vel.z = p.vel.z.add(dir.z.mul(push));
    clamp_air_speed(p);
}

/// **Steered while her Updraft blows** (2026-10-18, from play): for as long
/// as a column of hers stands and she is off the floor, the direction she
/// holds pushes her that way at `tuning::updraft_steer`, up to
/// `tuning::updraft_steer_cap` along it -- so the jump attack goes where she
/// leans, not only up.
fn steer_in_the_updraft(p: &mut Player, who: usize, input: Input, scene: &Scene) {
    if p.class != Class::Elementalist || p.grounded {
        return;
    }
    let (ax, az) = input.move_axis();
    if ax == 0 && az == 0 {
        return;
    }
    let blowing = scene
        .effects
        .iter()
        .flatten()
        .any(|e| e.kind == EffectKind::Updraft && e.owner as usize == who);
    if !blowing {
        return;
    }
    let dir = move_dir(input.aim_turns(), ax, az);
    let along = p.vel.x.mul(dir.x).add(p.vel.z.mul(dir.z));
    let room = t::updraft_steer_cap().sub(along);
    if room.raw() <= 0 {
        return;
    }
    let push = t::updraft_steer().mul(DT).min(room);
    p.vel.x = p.vel.x.add(dir.x.mul(push));
    p.vel.z = p.vel.z.add(dir.z.mul(push));
}

/// **The thrown shield's volume this frame**, in the same shape a swing's
/// is: a capsule along the stretch it flew this frame, as thick as the
/// shield. `None` unless it is in flight.
///
/// What a creature and a critter are struck by (`World::shields_strike_the_quarry`,
/// through [`part_under`] and `critter::Body::touched_by`, the two a swing
/// goes through) and what the overlay draws, so the drawing cannot drift from
/// the test. Along the frame's stretch rather than at the point it reached,
/// for the reason a bolt is (`bolt::step`): a fast throw does not step over a
/// knee-high body between two frames.
pub fn shield_hitbox(p: &Player) -> Option<Hitbox> {
    let Some(Shield::Flying {
        pos, vel, struck, ..
    }) = p.shield()
    else {
        return None;
    };
    Some(Hitbox {
        from: pos.sub(vel.scale(DT)),
        to: pos,
        radius: t::shield_radius(),
        flat: false,
        hits_crouching: true,
        unblockable: false,
        spent: struck,
        sweet: false,
    })
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
        // A disc is one point; a strip (the Reave's) is asked from its far
        // end, the sweet spot, back toward her.
        if box_out.from == box_out.to {
            return beast.part_struck(box_out.centre(), box_out.radius, attacker.hurt_height());
        }
        return (0..=3).rev().find_map(|i| {
            let at = crate::math::lerp3(box_out.from, box_out.to, Fx::ratio(i, 3));
            beast.part_struck(at, box_out.radius, attacker.hurt_height())
        });
    }
    const SAMPLES: i32 = 5;
    let thick = box_out.radius;
    let rig = beast.rig();
    // **The Dual mage's ball carries through to the weak point** (2026-10-13):
    // its flight crosses metres of a body a creature's size, and struck at the
    // first flank it brushed it never reached the part she threw it at -- the
    // Broodmother's sacs in the slam. So on the frame it first touches the
    // creature it strikes the softest part anywhere on the rest of its
    // flight, once, as the creature's own rule picks among parts one volume
    // touches. Where the rest of the flight is, is `moves::orb_at`'s.
    if let Action::Active { kind, left } = attacker.action
        && attacker.class == Class::DualMage
        && moves::dual::is_an_auto(kind)
        && !box_out.sweet
    {
        let mut touched = false;
        for i in 0..=SAMPLES {
            let at = crate::math::lerp3(box_out.from, box_out.to, Fx::ratio(i, SAMPLES));
            let foot = V3::new(at.x, at.y.sub(thick), at.z);
            touched |= beast
                .part_struck_on(&rig, foot, thick, thick.add(thick))
                .is_some();
        }
        if !touched {
            return None;
        }
        let m = moves::get(attacker.class, kind);
        let line = moves::orb_line(
            attacker.facing,
            attacker.aim_dir(),
            attacker.grounded,
            attacker.stoop,
        );
        let (was, _) = moves::orb_progress(&m, left);
        const ALONG: i32 = 12;
        let mut best: Option<usize> = None;
        for i in 0..=ALONG {
            let t = was.add(Fx::ONE.sub(was).mul(Fx::ratio(i, ALONG)));
            let at = moves::orb_at(attacker.pos, attacker.facing, line, &m, t);
            let foot = V3::new(at.x, at.y.sub(thick), at.z);
            if let Some(part) = beast.part_struck_on(&rig, foot, thick, thick.add(thick)) {
                best = Some(match best {
                    Some(seen)
                        if beast.sp().vulnerability(seen).raw()
                            >= beast.sp().vulnerability(part).raw() =>
                    {
                        seen
                    }
                    _ => part,
                });
            }
        }
        return best;
    }

    let mut best: Option<usize> = None;
    for i in 0..=SAMPLES {
        let at = crate::math::lerp3(box_out.from, box_out.to, Fx::ratio(i, SAMPLES));
        let foot = V3::new(at.x, at.y.sub(thick), at.z);
        if let Some(part) = beast.part_struck_on(&rig, foot, thick, thick.add(thick)) {
            // Softest wins, the creature's own rule: a swing that reaches a
            // leg with its haft and the ridge with its head has hit the ridge.
            best = Some(match best {
                Some(seen)
                    if beast.sp().vulnerability(seen).raw()
                        >= beast.sp().vulnerability(part).raw() =>
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

/// Is this fighter's dodge a blink?
///
/// The Blood mage's, and only while the flag is on. Grounded only: the airdodge
/// is already a commitment bought once per airtime, and a blink that also
/// worked off the floor would be the second jump this class has not earned.
fn blinks(p: &Player) -> bool {
    p.class == Class::BloodMage && t::dodge_blinks()
}

/// Arm the blink: the same haul machinery, pointed along the stick.
///
/// Reusing the haul rather than teleporting is what makes it drawable and
/// readable -- a body that is simply somewhere else cannot be reacted to and
/// cannot be animated -- and it means the two experiments on this class share
/// one piece of state instead of each growing their own. Four frames of smear
/// against a twenty-two-frame dodge, well inside the invulnerability.
///
/// **Stopped at the geometry.** `aim::first_along` against the arena and the
/// structures, the same question the Grasp's anchor asks, so a blink into a
/// wall arrives at the wall.
fn begin_blink(p: &mut Player, who: usize, dir: V3, scene: &aim::Scene) {
    let flat = V3::new(dir.x, Fx::ZERO, dir.z).normalized();
    let from = aim::origin(p.pos);
    let path = aim::Path {
        from,
        to: from.add(flat.scale(t::blink_range())),
    };
    let blocked = aim::first_along(
        path,
        t::body_radius(),
        who as u8,
        scene,
        aim::Targets::none().terrain().stones(),
    );
    let reach = match blocked {
        Some(c) => c.dist().sub(t::body_radius()),
        None => t::blink_range(),
    };
    if reach.raw() <= 0 {
        return;
    }
    // Back down to her feet: the ray was cast from her chest so that it meets
    // the same geometry an ability would, and where she *stands* is the floor
    // under the far end of it.
    let land = p.pos.add(flat.scale(reach));
    let frames = t::blink_frames();
    // **Constant time, not constant speed**, which is the opposite of the
    // Grasp's haul and correct for the opposite reason. The blink's distance is
    // whatever the wall allows, so a fixed speed would make a blink into a
    // corner a shorter move than a blink across the arena -- and how long she
    // is a smear for is the one thing the person opposite has to read it by.
    let speed = t::blink_range().div(Fx::from_int(frames as i32).mul(DT));
    // A frame in hand, so the last step lands her on the point rather than
    // leaving her a step short of it when the count runs out.
    begin_haul(
        p,
        aim::settle(land, scene.stones, scene.arena),
        speed,
        frames + 1,
    );
}

/// The velocity a Grasp haul is driving this frame, or `None` if she is not on
/// one.
///
/// Constant speed, like the Reaver's dash and for the same reason: the distance
/// was chosen by the ability -- the Grasp's reach and the length of the hold --
/// so a decaying pull would cover whatever distance the decay happens to be
/// tuned for rather than the distance the player aimed at. See
/// `Player::haul`.
///
/// Ends on arrival, and gravity and the arena are off while it runs, so the
/// line she travels is the line the arms went out along. Getting hit ends it,
/// because a commitment you can be hit out of and keep is not one.
fn step_haul(p: &mut Player) -> bool {
    if p.haul == 0 {
        return false;
    }
    p.haul -= 1;
    let gap = p.haul_to.sub(p.pos);
    let far = gap.len();
    let step = p.haul_speed.mul(DT);
    // **The frames ran out without arriving.** Nothing should get here -- the
    // bound is worked out from the distance and the speed -- so the answer is
    // to stop dead and let the world have her back rather than to snap her to a
    // point the pull never reached.
    if p.haul == 0 && far.raw() > step.raw() {
        p.vel = V3::ZERO;
        return false;
    }
    if far.raw() <= step.raw() {
        // **She lands on it, not near it**, which is the Reaver's dash arrival
        // word for word and for the same reason: the point was chosen as
        // somewhere a body can stand, and stopping half a metre short of it
        // leaves her inside whatever she was being pulled on to -- or, worse,
        // still carrying the pull's whole speed with nothing left to spend it
        // on. The first draft returned early here and let momentum finish the
        // job, and she sailed past the wall and oscillated around it.
        p.pos = p.haul_to;
        p.vel = V3::ZERO;
        p.haul = 0;
        return false;
    }
    p.vel = gap.scale(p.haul_speed.div(far));
    p.grounded = false;
    true
}

/// Arm a haul: where it is pulling to, and how fast.
///
/// The `frames` are a bound rather than the duration -- the drive ends on
/// arrival, and this only stops a haul running forever if something displaces
/// her mid-pull. Generous on purpose: a bound that cut a pull short would be a
/// second, invisible rule about how far a Grasp reaches.
fn begin_haul(p: &mut Player, to: V3, speed: Fx, frames: u16) {
    p.haul_to = to;
    p.haul_speed = speed;
    p.haul = frames;
    p.vel = V3::ZERO;
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
/// **Rebound**'s bounce: back off what the shield met, the way it was thrown,
/// and up.
fn bounce(p: &mut Player, along: V3) {
    let back = V3::new(along.x, Fx::ZERO, along.z)
        .normalized()
        .scale(t::rebound_kick());
    p.vel = V3::new(
        Fx::ZERO.sub(back.x),
        t::rebound_lift(),
        Fx::ZERO.sub(back.z),
    );
    p.grounded = false;
    // The bounce is the move's whole answer to the air: the hang an aerial
    // holds you in would bleed it away.
    p.air_stall = 0;
}

/// A Rebound whose shield met a body -- a fighter's, a creature's -- bounces
/// off it. The line `World::rebound` asks finds walls, stones and the floor;
/// a body is found by the blow itself.
fn rebound_off_a_body(p: &mut Player) {
    if p.class == Class::Bulwark
        && !p.grounded
        && p.action.attack_kind() == Some(moves::bulwark::REBOUND)
        && p.vel.y.raw() <= 0
    {
        let facing = p.facing;
        bounce(p, facing);
    }
}

/// The Bulwark's drive through a Battering ram's active frames, if he is in
/// one.
fn ramming(p: &Player) -> Option<V3> {
    if p.class != Class::Bulwark
        || !matches!(p.action, Action::Active { kind, .. } if kind == moves::bulwark::RAM)
    {
        return None;
    }
    Some(
        V3::new(p.facing.x, Fx::ZERO, p.facing.z)
            .normalized()
            .scale(t::ram_speed()),
    )
}

fn attack_step(p: &Player) -> Option<V3> {
    let kind = p.action.attack_kind()?;
    let mut m = moves::get(p.class, kind);
    m.step = reap_step(p, kind, m.step);
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

/// **How far the Reap lunges** (2026-10-18): half its row's step for a hold
/// of nothing, all of it for the longest (`Player::channelled`), and
/// `tuning::reap_ground_step` times that thrown from her ground. Every other
/// move's step is its own.
fn reap_step(p: &Player, kind: u8, step: Fx) -> Fx {
    if p.class != Class::BloodMage || kind != moves::blood::REAP {
        return step;
    }
    let half = Fx::ratio(1, 2);
    let held = half.add(half.mul(p.channelled.clamp(Fx::ZERO, Fx::ONE)));
    let ground = if p.thrown_at.raw() > Fx::ONE.raw() {
        t::reap_ground_step()
    } else {
        Fx::ONE
    };
    step.mul(held).mul(ground)
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
pub(crate) fn rushing(p: &Player) -> Option<V3> {
    match p.mechanic {
        Mechanic::Forms { rush, rush_vel, .. } if rush > 0 => Some(rush_vel),
        _ => None,
    }
}

/// Spend the charge and start the dash. False if there was no charge to spend.
///
/// **The dash goes where the crosshair is** (2026-10-13, from play): flat
/// along the floor toward the spot the crosshair is on, from his feet --
/// `aim::racing_path`, the line a crack races along, run by a body. It used
/// to go where the stick was held, so that it could be a retreat or a
/// sidestep, and it looked and felt like it was going somewhere he was not
/// pointing; turn the camera to retreat.
fn start_rush(p: &mut Player, who: usize, input: Input, scene: &Scene) -> bool {
    let Mechanic::Forms { recharge: 0, .. } = p.mechanic else {
        return false;
    };
    let dir = aim::racing_path(p.pos, who, input, t::rush_speed(), scene).dir();
    if let Mechanic::Forms {
        rush,
        recharge,
        rush_vel,
        fury,
        ..
    } = &mut p.mechanic
    {
        *rush = t::rush_frames();
        // A fresh run is braced afresh: nothing taken, the last frames' one
        // free blow unspent, the shoulder still to throw.
        *fury = class::Fury {
            braced: 0,
            eaten: false,
            shouldered: false,
            ..*fury
        };
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

/// **Following the shadow out** (2026-10-10): a forward dodge while the
/// shadow she just threw is still flying cuts the throw's remaining frames
/// short, so the dodge below is the dash, chasing it. Only once it has left
/// her -- the throw's active frames on -- and only while it is still going:
/// once it has landed, the dash is the ordinary one, pointed at it.
fn follow_the_shadow(p: &mut Player, input: Input) {
    let Some(kind) = p.action.attack_kind() else {
        return;
    };
    let thrown = matches!(p.action, Action::Active { .. } | Action::Recovery { .. });
    let (_, az) = input.move_axis();
    if p.class == Class::ShadowReaver
        && moves::reaver::places_the_shadow(kind)
        && thrown
        && input.has(Input::SHIFT)
        && az > 0
        && shadow::of(p).is_some_and(|s| matches!(s.doing, crate::class::Ghost::Casting { .. }))
        && (p.grounded || !p.air_dodged)
    {
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
    // **Before anything else**, and before `throw_move` steers the bars: what
    // this cast is worth is where she is standing right now, on the bar of the
    // force this move is made of. See [`Player::thrown_at`].
    p.thrown_at = worth_of(p, Some(kind));
    // **The Blood mage's vessel lets go.** A charge stored by a jump or a
    // dodge out of its hold comes out now, at the tier it reached, from
    // wherever she is: it was paid for when it was charged, so nothing is paid
    // again and there is no hold.
    if p.class == Class::BloodMage {
        // **A click comes up out of her ground** as its move, and that move
        // is the strong one: `thrown_at` was read above, on her ground.
        p.swim = if p.swim != SWIM_NONE && input.has(Input::SHIFT) {
            SWIM_SPENT
        } else {
            SWIM_NONE
        };
        // A Sweep strung from the last one keeps its place in the string;
        // anything else starts a new one.
        if p.string & STRINGING != 0 {
            p.string &= !STRINGING;
        } else {
            p.string = 0;
        }
        p.tier = 0;
        if p.vessel == kind {
            p.tier = p.vessel_tier.max(1);
            p.vessel = NO_VESSEL;
            p.vessel_tier = 0;
            lock_aim(p, who, kind, input, scene);
            return throw_move(p, kind, input, aerial);
        }
    }
    // Health is spent on the press, never on the hit. Missing is the
    // punishment, which is the whole of the Blood mage's economy -- see
    // `docs/design/kits/blood-mage.md`. Paid here even for a channelled move,
    // because the press is still the press: there is no way to cancel out of a
    // wind-up, so an ability you started is an ability you bought.
    pay(p, moves::get(p.class, kind).cost);
    // A move that charges **before** it starts does not start here. Pressing
    // the button opens the wind-up instead, and the move begins when the
    // button comes back up -- see `step_channel`. The other two charges come
    // after the startup and the press starts the move like any other.
    if moves::charge(p.class, kind) == Some(moves::Charge::TapOrHold) {
        aim_channel(p, who, kind, 0, input, scene);
        return Action::Channel { kind, held: 0 };
    }
    // The Air ball's hold is before the move too, and the press is where the
    // ball goes: on the floor under the crosshair, asked of `crate::aim` the
    // way Raise asks it. The world raises the ball there on this frame, and
    // the release sends it from there. See `World::roll_the_ball`.
    // The Blood mage's three charges: the hold is before the move, and each
    // tier is paid as it is reached. A tap is the first tier.
    if moves::charge(p.class, kind) == Some(moves::Charge::Tier) {
        p.tier = 1;
        aim_channel(p, who, kind, 0, input, scene);
        return Action::Channel { kind, held: 0 };
    }
    if moves::charge(p.class, kind) == Some(moves::Charge::Gather) {
        p.ball_at = aim::grounded_kept(who, input, moves::get(p.class, kind).reach, scene).to;
        aim_channel(p, who, kind, 0, input, scene);
        return Action::Channel { kind, held: 0 };
    }
    lock_aim(p, who, kind, input, scene);
    // **Gallows** (2026-10-18): a drive up, `tuning::gallows_drive` long,
    // through its wind-up and its cut -- at an angle when the crosshair is
    // level or low, exactly at it when it is higher (`aim::rise_dir`). From
    // the floor and in the air the same. It replaced the shadow thrown up,
    // the blink to it and the dive out of it.
    if p.class == Class::ShadowReaver && kind == moves::reaver::GALLOWS {
        let dir = aim::rise_dir(who, input, t::gallows_drive(), t::gallows_climb(), scene);
        let m = moves::get(p.class, kind);
        shadow::begin_rise(p, dir, m.startup.saturating_add(m.active));
        p.grounded = false;
        p.air_stall = 0;
    }
    // **The Plunge**: straight into the dive, along the line she picked. Her
    // drift is spent; `plunging` drives it from the next frame.
    if p.class == Class::ShadowReaver && kind == moves::reaver::PLUNGE {
        p.vel = V3::ZERO;
        p.air_stall = 0;
    }
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
    // A Strike not yet held is worth nothing over the pillar: the hold, if it
    // comes, sets this on release. A tap has to be the pillar as built.
    if moves::charge(p.class, kind) == Some(moves::Charge::Strike) {
        p.channelled = Fx::ZERO;
    }
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
    // The Dual mage's Veil starts its clock on the throw. A no-op for
    // everybody else.
    if p.class == Class::DualMage && kind == moves::dual::VEIL {
        dual::veil(p);
    }
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
    let m = moves::get(p.class, kind);
    let cap = m.channel;
    let charge = moves::charge(p.class, kind);
    // **The Blood mage's tap, waiting to be told**: let go inside the
    // Champion's window and it is the tap; held to its end, the field.
    if p.class == Class::BloodMage && moves::blood::held_into(kind).is_some() {
        if !input.has(moves::blood::click_of(kind)) {
            return Action::Active {
                kind,
                left: m.active,
            };
        }
        if m.startup + held + 1 >= t::hold_from() {
            return into_the_field(p, kind);
        }
        return Action::Channel {
            kind,
            held: held + 1,
        };
    }
    // **The Champion's finisher**: held long enough, counted from the press,
    // and the link is that weapon's finisher, aimed now as any move is on its
    // first frame.
    if charge == Some(moves::Charge::Heft) && input.has(channel_button(p.class, kind)) {
        if m.startup + held + 1 >= t::hold_from() {
            return finish_with(p, who, kind, input, scene);
        }
        return Action::Channel {
            kind,
            held: held + 1,
        };
    }
    // Held, and there is still room: wind on. At the cap it releases itself,
    // so a player holding the button through a fight is not quietly storing an
    // ability they have already paid for.
    // **A finisher** (2026-10-13): held to the end of the hold, at any
    // time, the Hemoplague, the spike and the Carpet each become their
    // finisher, and it spends all her grey on the frame it
    // begins (`Player::fuel`): the more there was, the bigger it is.
    if held >= cap && input.has(channel_button(p.class, kind)) {
        if let Some(big) = moves::blood::finisher_of(kind).filter(|_| p.class == Class::BloodMage) {
            p.fuel = p.grey.clamp(0, u16::MAX as i32) as u16;
            p.owed = p.grey.clamp(0, u16::MAX as i32) as u16;
            p.grey = 0;
            p.grey_rest = 0;
            p.tier = 0;
            lock_aim(p, who, big, input, scene);
            return throw_move(p, big, input, true);
        }
    }
    if input.has(channel_button(p.class, kind)) && held < cap {
        let held = held + 1;
        if charge == Some(moves::Charge::Tier) {
            // **A jump or a dodge out of the hold stores it** in her vessel,
            // at the tier it had reached, and she is free on this frame: the
            // jump below takes off from it. The charge and the movement take
            // turns rather than excluding each other.
            let (ax, az) = input.move_axis();
            if input.has(Input::SPACE) || (input.has(Input::SHIFT) && (ax != 0 || az != 0)) {
                p.vessel = kind;
                p.vessel_tier = p.tier.max(1);
                p.channelled = Fx::ratio(held.min(cap) as i32, cap.max(1) as i32);
                p.vessel_ready = false;
                p.tier = 0;
                return Action::Free;
            }
            // Each tier paid as it is reached, faster the more grey she has.
            let tier = tier_held(p, held);
            if tier > p.tier {
                pay(
                    p,
                    match tier {
                        2 => t::tier_two_cost(),
                        _ => t::tier_three_cost(),
                    },
                );
                p.tier = tier;
            }
        }
        aim_channel(p, who, kind, held, input, scene);
        return Action::Channel { kind, held };
    }
    // Released. The aim locks now, on the frame the wind-up becomes a move,
    // which is where every other move locks it too.
    aim_channel(p, who, kind, held, input, scene);
    // Released by the cap with the button still down: that press is spent,
    // and the button has to come up before it throws anything else.
    if p.class == Class::Elementalist && input.has(channel_button(p.class, kind)) {
        p.spent_clicks |= channel_button(p.class, kind);
    }
    match charge {
        // **The Dual mage's long holds** (2026-10-12): held to the end, the
        // spell comes out on its own; let go sooner and the click was a tap,
        // and the tap is thrown now -- the wing, or Binary or Phase. Nothing
        // of either had begun while it was undecided.
        Some(moves::Charge::TapOrHold) if held >= cap => throw_move(p, kind, input, true),
        Some(moves::Charge::TapOrHold) => match tap_of(p, kind).filter(|tap| !p.locked_out(*tap)) {
            Some(tap) => begin_move(p, who, tap, input, scene, true),
            None => Action::Free,
        },
        // The Strike: the startup already ran, and the hold was the pause
        // between it and the active frames. What it bought is how much of
        // the pillar's burn arrives at once, as a share of the longest hold.
        // Straight to the active frames -- the lockout, the echo and the
        // meter were all settled on the press.
        Some(moves::Charge::Strike) => {
            p.channelled = Fx::ratio(held.min(cap) as i32, cap.max(1) as i32);
            Action::Active {
                kind,
                left: m.active,
            }
        }
        // The crack, and the Grasp: what was bought is a distance, read back
        // off the path rather than recomputed from the hold. The two are the
        // same number -- `aim_channel` has just put the marker there -- and
        // taking it from the path is what makes the arms land on the marker
        // rather than on a second calculation that agrees with it today.
        //
        // `aerial` is always true here and always harmless: `arm_aerial`
        // returns on the spot for anybody whose feet are on something, and
        // aboard the creature they always are.
        // The tier is already on her (`Player::tier`), paid as it came; how
        // far into the hold she let go is how many spikes come up.
        Some(moves::Charge::Tier) => {
            p.channelled = Fx::ratio(held.min(cap) as i32, cap.max(1) as i32);
            throw_move(p, kind, input, true)
        }
        // The field was the move: let go, and she recovers. The Bramble
        // stands on at the size it reached (`step_plagues`).
        Some(moves::Charge::Field) => {
            p.lock_repeat(kind);
            Action::Recovery {
                kind,
                left: m.recovery,
            }
        }
        // The Champion's hold: under `hold_from` it was a press a little
        // long; up to `hold_full` it is a part charge, worth its share; full,
        // it is a different move -- the weapon's full charge for this place in
        // the string -- wound up from its own startup. Facing locks here, on
        // the release, as it does for every other move on its first frame.
        // Let go before the press was long enough: the link, as tapped.
        Some(moves::Charge::Heft) => Action::Active {
            kind,
            left: m.active,
        },
        // **The Reap**: how far into its hold she let go is how far it
        // lunges (`reap_step`).
        _ if p.class == Class::BloodMage && kind == moves::blood::REAP => {
            p.channelled = Fx::ratio(held.min(cap) as i32, cap.max(1) as i32);
            throw_move(p, kind, input, true)
        }
        _ => {
            p.channelled = p.aim_path.length();
            throw_move(p, kind, input, true)
        }
    }
}

/// **Pay for a cast, and owe it to the floor** (2026-10-13): `percent` of her
/// red spent, as every cost is, and what was actually paid remembered in
/// `Player::paid`, to land as a stain where the cast does. A miss is a setup.
fn pay(p: &mut Player, percent: i32) {
    let before = p.health;
    p.spend_health(percent);
    if p.class.wounds_go_grey() {
        p.paid = p.paid.saturating_add((before - p.health).max(0) as u16);
    }
}

/// Which tier a hold of `held` frames has reached: two at
/// `tuning::tier_two`, three at `tuning::tier_three`, sooner by up to
/// `tuning::charge_grey_faster` at a full bar of grey -- a wounded Blood mage
/// casts faster.
fn tier_held(p: &Player, held: u16) -> u8 {
    let faster = Fx::ONE.add(t::charge_grey_faster().sub(Fx::ONE).mul(p.grey_share()));
    let worth = Fx::from_int(held as i32).mul(faster).to_int();
    if worth >= t::tier_three() as i32 {
        3
    } else if worth >= t::tier_two() as i32 {
        2
    } else {
        1
    }
}

/// `Effect::banked` on a Black spike that is **a field of waves** -- the
/// spike's third tier. One is an eruption out of a stain; three is an area
/// that only hits (the Stake landing, the second tier's spikes).
pub const SPIKE_FIELD: i32 = 2;
/// `Effect::banked` on a Black spike that is **an eruption out of a stain**.
/// An eruption spends the stain it came out of and spills nothing: the same
/// spot hit again runs out rather than growing (2026-10-13).
pub const SPIKE_ERUPTION: i32 = 1;
pub const SPIKE_AREA: i32 = 3;

/// `Effect::banked` on a Hemoplague she is still holding, growing.
pub const GROWING: i32 = 1;
/// `Effect::banked` on a Black spike at or past this is an eruption **of
/// its own size** -- a Crimson cascade pool's, or the impact that starts one
/// -- whose blow is `banked - SPIKE_SCALED` percent of the spike's row.
pub const SPIKE_SCALED: i32 = 1000;
/// `Effect::banked` on a Black spike at or past this is **an eruption out of
/// a spike's own blow** (2026-10-19): `banked - SPIKE_ECHO` percent of the
/// spike's row, the share `tuning::spike_hit_erupts` makes of what it dealt.
/// Scaled as `SPIKE_SCALED` is, but no finisher.
pub const SPIKE_ECHO: i32 = 2000;
/// A whole, in percent: what `SPIKE_SCALED`'s share is counted in.
const PERCENT: i32 = 100;
/// `Effect::life` on a stain that is not lit: a lit stain's `life` counts the
/// frames until it erupts (`World::light_the_fuses`).
pub const UNLIT: u16 = u16::MAX;
/// On a thrall's `Effect::struck`: it stands on its owner's ground this
/// frame, and is the stronger for it -- what the renderer draws it by. The
/// top of the mask, above every (part, victim) bit an effect uses.
pub const ON_GROUND: usize = crate::effects::Struck::CAPACITY - 1;
/// `Player::swim`: on her feet.
pub const SWIM_NONE: u8 = 0;
/// `Player::swim`: sunk in her ground, swimming.
pub const SWIM_UNDER: u8 = 1;
/// `Player::swim`: come up with `shift` still down; nothing until it is let go.
pub const SWIM_SPENT: u8 = 2;
/// On `Player::string`: the press that strung the next Sweep, kept until the
/// move it starts reads it.
pub const STRINGING: u8 = 0x80;
/// On a thrall's `Effect::struck`: the hand it keeps to round a wall.
pub const LEFT_HAND: usize = crate::effects::Struck::CAPACITY - 2;

/// What a Black spike's hit is worth against its row's damage: an eruption
/// out of a stain is the bigger hit, a field's wave and the second tier's
/// spikes their own share, and the Stake's landing its row's number.
fn spike_share(effect: &Effect) -> Fx {
    if effect.banked >= SPIKE_ECHO {
        return Fx::ratio(effect.banked - SPIKE_ECHO, PERCENT);
    }
    if effect.banked >= SPIKE_SCALED {
        return Fx::ratio(effect.banked - SPIKE_SCALED, PERCENT);
    }
    match effect.banked {
        SPIKE_FIELD => t::spike_field_power(),
        SPIKE_AREA if effect.slot == moves::blood::BLACK_SPIKE => t::spike_two_power(),
        SPIKE_AREA => Fx::ONE,
        _ => t::erupt_damage(),
    }
}

/// What a burst of her blood is worth: the Tides' tier, the number an
/// exploding stain carries, or the row's own.
fn nova_blow(effect: &Effect) -> i32 {
    effect.damage()
}

/// What a tier is worth over the first, on the blow.
pub fn tier_power(tier: u8) -> Fx {
    match tier {
        3 => t::tier_three_power(),
        2 => t::tier_two_power(),
        _ => Fx::ONE,
    }
}

/// Which button holds a channel open. The slot's own, since a channel is the
/// front of a move rather than a thing of its own.
const fn channel_button(class: Class, kind: u8) -> u16 {
    // The Dual mage's long holds are the click each follows: the dark wing's
    // held on, the light wing's, and the middle click from its press.
    if matches!(class, Class::DualMage) {
        match kind {
            moves::dual::CHAIN_LIGHTNING => return moves::dual::keys::DARK,
            moves::dual::DISK_RAY => return moves::dual::keys::LIGHT,
            moves::dual::FLURRY_DASH => return moves::dual::keys::TWILIGHT,
            _ => {}
        }
    }
    if matches!(class, Class::Champion) {
        return match moves::champion::weapon(kind) {
            moves::champion::SWORD => Input::LEFT,
            moves::champion::HAMMER => Input::MIDDLE,
            _ => Input::RIGHT,
        };
    }
    match kind {
        // The Elementalist's three holds are her three clicks: Fissure is the
        // earth click held (the stone was raised on the press and the crack
        // is what letting go throws), the Strike is the fire click held, and
        // the Air ball is the wind click held.
        SLOT_COMMITTED if matches!(class, Class::Elementalist) => moves::elementalist::keys::EARTH,
        SLOT_SPECIAL if matches!(class, Class::Elementalist) => moves::elementalist::keys::FIRE,
        // The Blood mage's three charges are her three clicks held.
        _ if matches!(class, Class::BloodMage) => moves::blood::click_of(kind),
        moves::elementalist::AIR_BALL if matches!(class, Class::Elementalist) => {
            moves::elementalist::keys::WIND
        }
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
    // The Strike is aimed at the point the crosshair is on, exactly as the
    // untouched pillar is: live through the hold, and the hold buys nothing
    // about *where*. The other two buy a distance along the solved line.
    // The Air ball's hold buys size, not distance: its line is only ever
    // read for the way it rolls.
    // A field is laid where the crosshair is when the hold opens, and grows
    // there: its marker is that spot, not a point walking out with the hold.
    if !matches!(
        moves::charge(p.class, kind),
        Some(
            moves::Charge::Strike
                | moves::Charge::Gather
                | moves::Charge::Tier
                | moves::Charge::Field
                | moves::Charge::TapOrHold
        )
    ) {
        p.aim_path.to = p.aim_path.at(m.reach_after(held));
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
    aim_at(p, who, kind, aimed_reach(p.class, kind), input, scene);
}

/// How far along the crosshair's line a move is aimed: its own reach, but for
/// the one whose reach and line are two things -- the Crescent, a sword cut
/// whose blade of light flies `tuning::crescent_reach`. Aimed at the cut's
/// arm's length, its line would climb from the chest to a point an arm's
/// length off the eye and fly over everybody's head.
fn aimed_reach(class: Class, kind: u8) -> Fx {
    if class == Class::Champion && kind == moves::champion::CRESCENT {
        t::crescent_reach()
    } else {
        moves::get(class, kind).reach
    }
}

/// The three lines themselves, at whatever reach the caller has decided on.
/// Only a channel has anything to decide: every other move is aimed at the
/// reach in its own row.
fn aim_at(p: &mut Player, who: usize, kind: u8, reach: Fx, input: Input, scene: &Scene) {
    let m = moves::get(p.class, kind);
    p.stoop = Fx::ZERO;
    p.aim_path = match m.aim() {
        // Her stones stay at her level when she did not point down
        // (`aim::grounded_kept`); everybody else's are the plain rule, and so
        // is her Fire pillar (2026-10-21, from play): fire is not footing, and
        // from a ledge she means the floor below it.
        aim::Kind::Grounded if p.class == Class::Elementalist && kind != SLOT_SPECIAL => {
            aim::grounded_kept(who, input, reach, scene)
        }
        aim::Kind::Grounded => aim::grounded_path(who, input, reach, scene),
        aim::Kind::Skillshot => aim::skillshot_path(who, input, reach, scene),
        aim::Kind::Overhead => aim::overhead_path(who, input, reach, scene),
        // Not aimed at anything -- a body moving. What it commits to is the
        // **plane** it swings in: the yaw is the facing, which is locked
        // already, and the pitch is the rest of the same look, dead-zoned so
        // that looking slightly down at somebody does not tilt the swing into
        // the floor. The Champion's weapons read the plane; a disc-shaped
        // swing only reads the direction. The dead zone is a standing rule:
        // off the ground you are above what you are hitting, and the swing
        // follows the camera the whole way.
        aim::Kind::Swing => {
            // How tall the thing under the crosshair is: what a standing
            // swing dips to meet when it is short. Only asked standing, and
            // only in a fight with small bodies in it -- everywhere else the
            // answer is a fighter's height and the swing is what it was.
            let stands = if p.grounded && scene.critters.any() {
                aim::stands_at(who, input, scene)
            } else {
                aim::Stand::fighter()
            };
            p.stoop = aim::stoop(input, p.grounded, stands);
            let up = aim::underfoot_up(who, scene);
            aim::swing_path(
                p.pos, p.facing, input, p.grounded, reach, m.hand, stands, up,
            )
        }
        aim::Kind::AtTheMechanic => aim::mechanic_path(p.pos, &p.mechanic),
        // From the stone she is holding churning, flat toward the crosshair's
        // spot on the ground. Her own feet when there is none, which the
        // crack's slot cannot reach without one -- see `hold_the_churn`.
        // The Air ball races the same way from where it was raised.
        aim::Kind::Racing => {
            let from = if p.class == Class::Elementalist && kind == moves::elementalist::AIR_BALL {
                p.ball_at
            } else {
                held_stone_at(p).unwrap_or(p.pos)
            };
            aim::racing_path(from, who, input, reach, scene)
        }
    };
    // **A spike aimed through one of her pools goes to the pool**
    // (2026-10-14): it stands a body's height at most, and the crosshair on
    // that column is the crosshair on it (`aim::column_on_sight`), with the
    // blink's slack (`tuning::pool_lock`) above it.
    if p.class == Class::BloodMage && moves::blood::seeks_pools(kind) {
        let columns = scene
            .effects
            .iter()
            .flatten()
            .filter(|e| e.is_a_pool() && e.owner == who as u8)
            .map(|e| {
                (
                    e.pos,
                    e.pool_radius().add(t::body_radius()),
                    e.pool_height().add(t::pool_lock()),
                )
            });
        if let Some(to) = aim::column_on_sight(who, input, reach, columns, scene) {
            p.aim_path.to = to;
        }
    }
}

/// Where the stone the Elementalist is holding churning stands, if she is.
fn held_stone_at(p: &Player) -> Option<V3> {
    let Mechanic::Structures(slots) = p.mechanic else {
        return None;
    };
    slots
        .get(p.charging_stone as usize)
        .copied()
        .flatten()
        .map(|s| s.at)
}

/// The mechanic key held past the rise: the Elementalist's stone stays
/// churning and the hold becomes Fissure's.
///
/// Three cases, read every frame she is holding a stone:
///
/// - **Free, key down, and the stone is about to erupt.** The rise was the
///   tap window and the tap has become a hold: she goes into the crack's
///   channel and the stone stays on the churn's last frame.
/// - **Free, key up.** A tap. The stone erupts as it always did and there is
///   nothing to hold.
/// - **In the channel, or in Fissure's startup after letting go.** The stone
///   is held churning until the crack comes out -- `advance` clears the hold
///   on the crack's first active frame. Anything else -- a hit, a different
///   move, a dodge -- ends the hold, and the stone erupts where it stands.
///
/// Runs after `step_mechanic` and before the countdown, on a frame the
/// stones have already aged, so the wind-back is what the rest of the frame
/// sees.
fn hold_the_churn(p: &mut Player, input: Input) {
    if p.class != Class::Elementalist || p.charging_stone == NO_STONE {
        return;
    }
    let Mechanic::Structures(mut slots) = p.mechanic else {
        p.charging_stone = NO_STONE;
        return;
    };
    let slot = p.charging_stone as usize;
    let Some(stone) = slots.get_mut(slot).and_then(|s| s.as_mut()) else {
        p.charging_stone = NO_STONE;
        return;
    };
    let holding = input.has(moves::elementalist::keys::EARTH);
    match p.action {
        Action::Free if holding && p.grounded => {
            if stone.about_to_erupt() {
                stone.hold_churning();
                p.action = Action::Channel {
                    kind: SLOT_COMMITTED,
                    held: 0,
                };
            }
        }
        Action::Channel { kind, .. } | Action::Startup { kind, .. } if kind == SLOT_COMMITTED => {
            stone.hold_churning();
        }
        _ => {
            p.charging_stone = NO_STONE;
            return;
        }
    }
    p.mechanic = Mechanic::Structures(slots);
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
    // **Each successive hang in one airtime is worth less** -- see
    // `Player::air_stalls`. The first is the move's own number; after that the
    // falloff compounds, so a pair of pokes alternated forever is a slow
    // descent rather than a hover, and the sum of everything the air will give
    // you is bounded before you touch the floor.
    let full = moves::get(p.class, kind).air_stall;
    if full > 0 {
        let mut worth = Fx::from_int(full as i32);
        // The Fire carpet's falls off by the carpets before it alone, and
        // steeply: see `Player::carpets`.
        let carpet = p.class == Class::Elementalist && kind == moves::elementalist::FIRE_CARPET;
        let (before, falloff) = if carpet {
            (p.carpets, t::carpet_hang_falloff())
        } else {
            (p.air_stalls, t::air_stall_falloff())
        };
        for _ in 0..before {
            worth = worth.mul(falloff);
        }
        if carpet {
            p.carpets = p.carpets.saturating_add(1);
        }
        // And the Reaver's Fan wears down on its own as she keeps throwing it,
        // landing or not. See `shadow::fan_hang`.
        if p.class == Class::ShadowReaver && kind == moves::reaver::FAN {
            worth = shadow::fan_hang(p, worth);
        }
        p.air_stall = worth.to_int().max(0) as u16;
        p.air_stalls = p.air_stalls.saturating_add(1);
    } else if !(p.class == Class::Elementalist && kind == moves::elementalist::UPDRAFT) {
        // A move with no hang of its own ends one still running -- except the
        // Updraft, which only ever pushes up: the Fire carpet's hang carries
        // her through its wind-up, so the column comes up through the carpet
        // rather than a fall's height below it (2026-10-21, from play).
        p.air_stall = 0;
    }

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
        || (p.class == Class::Elementalist && kind == moves::elementalist::AIR_BOLT)
        // And the Bulwark's Rebound is his Bash in the air (2026-10-10).
        || (p.class == Class::Bulwark && kind == moves::bulwark::REBOUND);
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
/// Air control at knockback speed: it turns you, it never slows you.
///
/// `air_accelerate` ends on the air speed cap, a rule about how fast a fighter
/// may make *themselves* go. Knockback routinely exceeds it, so left alone,
/// holding any direction during a launch would cut the hit short. Pressing a
/// direction must never be worth less knockback than pressing nothing, so the
/// flat speed is put back along the new heading; below the cap this does
/// nothing at all.
fn steer_while_stunned(p: &mut Player, wish: V3, wish_speed: Fx) {
    let before = V3::new(p.vel.x, Fx::ZERO, p.vel.z).flat_len();
    air_accelerate(p, wish, wish_speed);
    let after = V3::new(p.vel.x, Fx::ZERO, p.vel.z).flat_len();
    if after.raw() > 0 && before.raw() > after.raw() {
        let restore = before.div(after);
        p.vel.x = p.vel.x.mul(restore);
        p.vel.z = p.vel.z.mul(restore);
    }
}

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
    // Only the Elementalist's stones are placed by this; they stay at her
    // level when she did not point down (`aim::grounded_kept`).
    let placed = |reach| aim::grounded_kept(who, input, reach, scene).to;
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
                // Slower the more it holds: see `bulwark::flight_speed`.
                Shield::Held { weight } => Shield::Flying {
                    pos: from,
                    vel: aim::skillshot_path(who, input, t::shield_range(), scene)
                        .dir()
                        .scale(bulwark::flight_speed(weight)),
                    outbound: true,
                    travelled: Fx::ZERO,
                    weight,
                    struck: false,
                    met: Met::NONE,
                },
                Shield::Planted { pos, weight } => {
                    let to_owner = from.sub(pos);
                    Shield::Flying {
                        pos,
                        vel: to_owner.normalized().scale(bulwark::flight_speed(weight)),
                        outbound: false,
                        travelled: Fx::ZERO,
                        weight,
                        struck: false,
                        met: Met::NONE,
                    }
                }
                // **The leap and the shield meet in the middle**, since
                // 2026-09-23. He is thrown toward it and it turns and comes
                // back to meet him, homing like a recall, so he arrives with
                // it in hand and in the air -- which is what makes "throw,
                // leap, slam" a thing he can do. Before, the leap was slower
                // than the throw and fell short of it every time, and every
                // Bulwark move needs the shield in hand, so the slam out of a
                // leap the kit promised had no way to happen. See
                // `bulwark-v2.md` and the feel log.
                Shield::Flying {
                    pos,
                    travelled,
                    weight,
                    ..
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
                        vel: from
                            .sub(pos)
                            .normalized()
                            .scale(bulwark::flight_speed(weight)),
                        outbound: false,
                        travelled,
                        weight,
                        struck: false,
                        met: Met::NONE,
                    }
                }
            });
        }

        // Rush. It used to cycle the weapon form, which was the mode this
        // rebuild deleted -- the weapon is a mouse button now, so the mechanic
        // key is free for the mechanic the class is actually built around.
        Mechanic::Forms { .. } => {
            start_rush(p, who, input, scene);
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
            // Remembered for the hold: the key still down past this stone's
            // rise is Fissure's charge. See `hold_the_churn`.
            let raised = stones::raise(p, class::Structure::raised(placed(t::raise_reach())));
            p.charging_stone = raised.map_or(NO_STONE, |slot| slot as u8);
        }

        // Health is the resource; there is no separate button.
        Mechanic::Blood => {}

        // The meter is steered by which button attacks, not by this one.
        Mechanic::Meter { .. } => {}
    }
}

/// Per-tick mechanic upkeep: shields in flight, shadows on a clock, the meter
/// burning at depth.
fn step_mechanic(p: &mut Player, arena: &Terrain) {
    bulwark::drain(p);
    match p.mechanic {
        Mechanic::Shield(Shield::Flying {
            pos,
            vel,
            outbound,
            travelled,
            weight,
            struck,
            met,
        }) => {
            let step = vel.scale(DT);
            let next = pos.add(step);
            let gone = travelled.add(step.flat_len());
            p.mechanic = Mechanic::Shield(if outbound {
                if gone.raw() >= t::shield_range().raw() {
                    // **Planted means standing on the floor** under where it
                    // stopped, since 2026-09-23. It used to stay where its
                    // flight ended -- at hand height on a level throw -- which
                    // was harmless while a planted shield was a marker, and is
                    // a wall everybody walks under now that it is a solid.
                    // See `bulwark::wall`.
                    Shield::Planted {
                        pos: V3::new(next.x, arena.ground_under(next), next.z),
                        weight,
                    }
                } else {
                    Shield::Flying {
                        pos: next,
                        vel,
                        outbound,
                        travelled: gone,
                        weight,
                        struck,
                        met,
                    }
                }
            } else {
                let hand = aim::origin(p.pos);
                if next.sub(hand).flat_len().raw() < Fx::ONE.raw() {
                    Shield::Held { weight }
                } else {
                    // Home in, so the return does not miss a moving owner.
                    let dir = hand
                        .sub(next)
                        .normalized()
                        .scale(bulwark::flight_speed(weight));
                    Shield::Flying {
                        pos: next,
                        vel: dir,
                        outbound,
                        travelled: gone,
                        weight,
                        struck,
                        met,
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
            let live = p.action.attack_kind().is_some_and(|kind| {
                moves::champion::in_string(kind) || moves::champion::is_full(kind)
            });
            // Free, on the floor or off it: a jump no longer ends a string
            // (2026-10-13), since a jump is how the next hit of one arrives.
            let standing = p.action.actionable();
            // **The window opens when he may act again** (2026-10-20): at a
            // landed link's earliest cancel, still in its recovery, so it is
            // the same window -- and the Beat the same place in it -- for
            // every weapon.
            let cancellable = may_cancel(p);
            let stunned = p.action.stunned();
            let charging = winding_a_hold(p);
            if let Mechanic::Forms {
                rush,
                recharge,
                chain,
                chain_left,
                chain_hit,
                takeoff,
                fury,
                ..
            } = &mut p.mechanic
            {
                *rush = if stunned { 0 } else { rush.saturating_sub(1) };
                *recharge = recharge.saturating_sub(1);
                *takeoff = takeoff.saturating_sub(1);
                if *chain == 0 {
                    *chain_left = 0;
                } else if live && !cancellable {
                    *chain_left = t::chain_grace();
                } else if standing || cancellable {
                    // Past the third, too (2026-10-20, from play): the
                    // window runs on, and the next string's first hit keeps
                    // the beat. A press throws that first hit.
                    *chain_left = chain_left.saturating_sub(1);
                } else {
                    // The fighter is doing something a string does not
                    // survive.
                    *chain_left = 0;
                }
                if *chain_left == 0 {
                    *chain = 0;
                    *chain_hit = false;
                    fury.early = false;
                    fury.on_beat = false;
                }
                if !live {
                    fury.true_link = false;
                }
                // The meter holds after a landed hit, then drains -- and
                // holds still while he winds up a hold (2026-10-13, from
                // play): a finisher started on a full meter is a finisher on
                // a full meter, not one that watched it run down.
                if charging {
                    // Held where it is.
                } else if fury.momentum_left > 0 {
                    fury.momentum_left -= 1;
                } else {
                    fury.momentum = (fury.momentum as i32 - t::momentum_drain()).max(0) as u16;
                }
            }
        }

        // The second body: where it is, what it is copying, and its clock.
        // See `crate::shadow`.
        Mechanic::Shadow(_) => shadow::step(p),

        // Two bars and the hill between them: calm, the drift, the burn, and
        // the clock while she is ascending. See `crate::dual`.
        // The spellbook's own clocks run here too; the frame ascension
        // begins is read off the body by the world, for Apotheosis.
        Mechanic::Meter { .. } => {
            dual::step(p);
        }

        _ => {}
    }
}

// ---------------------------------------------------------------------------
// The depth curve
// ---------------------------------------------------------------------------
//
// The Dual mage's founding idea: **power scales continuously with the bar of
// the force a move is made of.** Not thresholds, not two versions of a move
// with a snap between them -- a straight line from `tuning::depth_floor` at
// empty to `tuning::depth_ceiling` at full, with every point on it reachable.
// Which of her two bars is `dual::depth_at`'s to answer: a cast reads the one
// she is carrying, an auto its own.
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
    // Mid-move, it is what the bars said when she committed. Throwing anything
    // moves a bar on the press and the finisher moves it a long way, so a
    // cast read live would be worth its own push -- see [`Player::thrown_at`].
    match p.action.attack_kind() {
        Some(_) => p.thrown_at,
        None => live_depth(p),
    }
}

/// The curve read off the bars as they are this instant, for whatever she is
/// carrying. See [`dual::depth_at`].
fn live_depth(p: &Player) -> Fx {
    worth_of(p, None)
}

/// **What a cast is worth, as a multiplier on what it deals**, for whatever
/// `kind` she is throwing: the Dual mage's depth on her bars, and **the
/// Blood mage's strong version** on her own ground (2026-10-18,
/// `tuning::ground_power`). One for everybody else.
fn worth_of(p: &Player, kind: Option<u8>) -> Fx {
    if p.class == Class::BloodMage {
        return if p.on_blood {
            t::ground_power()
        } else {
            Fx::ONE
        };
    }
    dual::depth_at(p, kind)
}

/// **Does the Blood mage sink into her ground** on this frame's `shift`?
/// On the floor, on her ground, not already under or waiting for `shift` to
/// come up, and with no stain under the crosshair -- the teleport comes
/// first, exactly as before.
fn dives_in(p: &Player, who: usize, input: Input, scene: &Scene) -> bool {
    p.class == Class::BloodMage
        && input.has(Input::SHIFT)
        && p.grounded
        && p.on_blood
        && p.swim == SWIM_NONE
        && pool_under_the_crosshair(p, who, input, scene).is_none()
}

/// **Swimming, one frame** (2026-10-18). Under, she stays under while `shift`
/// is held and she is free and on the floor; let go, a jump, a hit or a move
/// brings her up. **Off the edge of her ground she pops out**, along the way
/// she was going at `tuning::swim_pop`. Come up with `shift` still down, she
/// waits for it to be let go before it means anything again.
fn swim_or_surface(p: &mut Player, input: Input) {
    if p.swim == SWIM_SPENT && !input.has(Input::SHIFT) {
        p.swim = SWIM_NONE;
    }
    if p.swim != SWIM_UNDER {
        return;
    }
    let held = input.has(Input::SHIFT);
    if !held || !p.grounded || !p.action.actionable() || p.health <= 0 {
        p.swim = if held { SWIM_SPENT } else { SWIM_NONE };
        return;
    }
    if !p.on_blood {
        p.swim = SWIM_SPENT;
        let flat = V3::new(p.vel.x, Fx::ZERO, p.vel.z);
        if flat.flat_len().raw() > 0 {
            let frames = (t::dodge_frames() / 2).max(1);
            let reach = t::swim_pop().mul(DT).mul(Fx::from_int(frames as i32));
            let to = p.pos.add(flat.normalized().scale(reach));
            begin_haul(p, to, t::swim_pop(), frames + 1);
        }
    }
}

/// **The Sweep strung** (2026-10-18): right click pressed again inside a
/// Sweep's recovery ends it, and the next Sweep is the next blow of the
/// string -- the second back the other way, the third heavier -- up to
/// three. Its own press edge, kept on the fighter as the Reaver keeps hers.
fn string_the_sweep(p: &mut Player, input: Input) {
    if p.class != Class::BloodMage {
        return;
    }
    let click = moves::blood::keys::SCYTHE;
    let pressed = input.has(click) && !p.right_held;
    p.right_held = input.has(click);
    if !pressed {
        return;
    }
    let place = p.string & !STRINGING;
    if let Action::Recovery { kind, .. } = p.action
        && kind == moves::blood::SWEEP
        && place < 2
    {
        p.string = (place + 1) | STRINGING;
        p.action = Action::Free;
    }
}

/// Which blow of a string this Sweep is, if it is one: nought, one or two.
fn string_place(p: &Player, kind: u8) -> Option<u8> {
    (p.class == Class::BloodMage && kind == moves::blood::SWEEP).then_some(p.string & !STRINGING)
}

/// **What a swing of the Blood mage's is worth beside its row** (2026-10-18):
/// the third Sweep of a string at `tuning::sweep_third`, and the Harvest at
/// its finisher's blow -- the row and `finisher_per_grey` for every point of
/// grey it spent (`Player::fuel`). One for everything else.
fn blood_blow(p: &Player, kind: u8) -> Fx {
    if string_place(p, kind) == Some(2) {
        return t::sweep_third();
    }
    if p.class == Class::BloodMage && kind == moves::blood::HARVEST {
        let m = moves::get(p.class, kind);
        let extra = Fx::from_int(p.fuel as i32).mul(t::finisher_per_grey());
        return Fx::ONE.add(extra.div(Fx::from_int(m.damage.max(1))));
    }
    Fx::ONE
}

/// What a swing that dealt `dealt` heals its thrower: the row's leech for
/// every class, and **the Harvest pays back the grey it burned**, as every
/// finisher of hers does (`tuning::finisher_repays`, `Player::owed`).
fn swing_leech(p: &Player, kind: u8, dealt: i32) -> i32 {
    if p.class == Class::BloodMage && kind == moves::blood::HARVEST {
        return dealt.max(0) * t::finisher_repays() / 100;
    }
    moves::get(p.class, kind).leeched(dealt)
}

/// Is the Dual mage off the floor: at the three-quarter tier, or ascended?
///
/// **`false` for every other class**, so it is safe to ask of anybody.
///
/// The float used to trigger at *depth*, a position on the one signed bar. The
/// two bars have no such position, so it lives on the same tier the second jump
/// and the slow fall do -- the lower bar at three quarters, `tuning::tier_jump`
/// -- deliberately: that tier is already "her feet stop obeying the floor", and
/// one threshold is what makes the float, the second jump and the slow fall
/// three faces of the same decision rather than three rules that happen to fire
/// near each other. The burn is what riding both bars up costs; this is one of
/// the things it buys. See `dual::Tier`.
///
/// Read live rather than from `thrown_at`, unlike [`depth`]. It is not what a
/// cast is worth -- it is where she *is*, and where she is has to match what
/// the renderer is drawing on the frame it draws it.
pub fn floating(p: &Player) -> bool {
    dual::ascending(p) || dual::tier(p) >= dual::Tier::Jump
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
    // **The majors grow along a steeper line** (2026-10-12): Judgement's
    // middle column is read off the same `dual::major_size` its outer ones and
    // its field are, from the power it was thrown with.
    if moves::dual::is_the_finisher(kind) {
        return dual::major_size(p.thrown_at);
    }
    crate::math::lerp(Fx::ONE, depth(p), t::depth_size())
}

/// Attacking goads one of the Dual mage's bars. Nothing else moves them up, so
/// every step is a consequence of a decision the player made. See
/// [`dual::steer`].
fn steer_meter(p: &mut Player, kind: u8) {
    dual::steer(p, kind);
}

fn hash_mechanic(h: &mut Fnv, m: &Mechanic) {
    match m {
        Mechanic::Shield(Shield::Held { weight }) => {
            h.write_u32(0);
            h.write_i32(weight.raw());
        }
        Mechanic::Shield(Shield::Planted { pos, weight }) => {
            h.write_u32(1);
            hash_v3(h, pos);
            h.write_i32(weight.raw());
        }
        Mechanic::Shield(Shield::Flying {
            pos,
            vel,
            outbound,
            travelled,
            weight,
            struck,
            met,
        }) => {
            h.write_u32(2);
            hash_v3(h, pos);
            hash_v3(h, vel);
            h.write_u32(*outbound as u32);
            h.write_i32(travelled.raw());
            h.write_i32(weight.raw());
            // One word for both: a flight that has met no creature hashes as
            // it did before there was anything to remember, and one that
            // met what sixteen bits could name hashes as it did then.
            h.write_u32(*struck as u32 | met.compat() << 1);
            if met.compat() == u32::MAX >> 1 {
                for &w in met.critters.words().iter().chain(met.creatures.words()) {
                    h.write_u32(w);
                }
            }
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
            rise_used,
            lifted,
            fury,
            liquid,
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
            h.write_u32(*rise_used as u32);
            h.write_u32(*lifted as u32);
            h.write_u32(fury.braced as u32);
            h.write_u32(
                fury.eaten as u32
                    | (fury.shouldered as u32) << 1
                    | (fury.true_link as u32) << 3
                    | (fury.clicks as u32) << 8
                    | (fury.early as u32) << 16
                    | (fury.on_beat as u32) << 17
                    | (fury.fresh as u32) << 20
                    | (fury.may_hold as u32) << 24
                    | (fury.place as u32) << 25,
            );
            h.write_u32(fury.momentum as u32 | (fury.momentum_left as u32) << 16);
            h.write_u32(
                fury.last_weapon as u32 | (fury.spent as u32) << 8 | (fury.clash_by as u32) << 16,
            );
            h.write_u32(fury.fresh_left as u32 | (fury.landed as u32) << 16);
            h.write_u32(liquid.raw());
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
                    for &w in struck.words() {
                        h.write_u32(w);
                    }
                }
            }
            h.write_u32(shadow.echo as u32);
            h.write_u32(shadow.echo_age as u32);
            h.write_u32(shadow.echo_used as u32);
            h.write_u32(shadow.dash as u32);
            h.write_u32(shadow.carry as u32);
            hash_v3(h, &shadow.lunge);
            h.write_u32(shadow.jump_banked as u32);
            h.write_u32(shadow.refused as u32);
            h.write_u32(shadow.shift_spent as u32);
            // Hashed only when the clocks are running, so the replays
            // pinned before they existed keep their checksums.
            if shadow.hang != 0 || shadow.flip != 0 || shadow.collected {
                h.write_u32(shadow.hang as u32);
                h.write_u32(shadow.flip as u32);
                h.write_u32(shadow.collected as u32);
            }
            // The same, for the clocks of 2026-10-18.
            if shadow.rest != 0 || shadow.rising != 0 || shadow.fans != 0 || shadow.fan_clock != 0 {
                h.write_u32(shadow.rest as u32);
                hash_v3(h, &shadow.rise);
                h.write_u32(shadow.rising as u32);
                h.write_u32(shadow.fans as u32);
                h.write_u32(shadow.fan_clock as u32);
            }
        }
        Mechanic::Structures(slots) => {
            h.write_u32(5);
            for slot in slots {
                match slot {
                    Some(s) => {
                        hash_v3(h, &s.at);
                        hash_v3(h, &s.vel);
                        h.write_u32(s.age as u32);
                        for &w in s.struck.words() {
                            h.write_u32(w);
                        }
                        h.write_u32(s.launched as u32);
                        hash_v3(h, &s.launch_from);
                        for &w in s.knock_struck.words() {
                            h.write_u32(w);
                        }
                        h.write_u32(s.rise as u32);
                        hash_v3(h, &s.erupt);
                        h.write_i32(s.scale.raw());
                        h.write_u32(s.lit as u32);
                    }
                    None => h.write_u32(0),
                }
            }
        }
        Mechanic::Blood => h.write_u32(6),
        Mechanic::Meter {
            dark,
            light,
            colour,
            ascending,
            jumped,
            hopped,
            dash_jump,
            landed,
            singe,
            veil,
            warded,
        } => {
            h.write_u32(7);
            h.write_i32(dark.raw());
            h.write_i32(light.raw());
            h.write_u32(*colour as u32);
            h.write_u32(*ascending as u32);
            h.write_u32(*jumped as u32);
            h.write_u32(*landed as u32);
            h.write_i32(singe.raw());
            // The spellbook's, 2026-10-12: only when any is set, so a mage
            // who has thrown nothing hashes as she did.
            if *veil != 0 || *warded {
                h.write_u32(*veil as u32);
                h.write_u32(*warded as u32);
            }
            if *hopped {
                h.write_u32(1);
            }
            if *dash_jump {
                h.write_u32(2);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// The checksum's pieces. One function per kind of thing, so the whole-world
// hash (`World::digest`) and a region's (`World::region_checksum`) write the
// same bytes for the same body and cannot drift apart. See
// `docs/design/regions.md` §"A region checksum".
// ---------------------------------------------------------------------------

fn hash_bolt(h: &mut Fnv, b: &crate::bolt::FireBolt) {
    h.write_u32(b.owner as u32 + 1);
    hash_v3(h, &b.pos);
    hash_v3(h, &b.dir);
    h.write_i32(b.travelled.raw());
}

fn hash_shard(h: &mut Fnv, d: &crate::debris::Shard) {
    h.write_u32(d.owner as u32 + 1);
    hash_v3(h, &d.pos);
    hash_v3(h, &d.dir);
    h.write_i32(d.travelled.raw());
}

fn hash_gust(h: &mut Fnv, g: &crate::gust::Gust) {
    h.write_u32(g.owner as u32 + 1);
    // Which of the two, on the wire: a bolt and a disc at the
    // same place going the same way are not the same shot, and
    // a checksum that could not tell them apart would let one
    // peer's disc be the other's bolt.
    h.write_u32(g.gale as u32);
    hash_v3(h, &g.pos);
    hash_v3(h, &g.dir);
    h.write_i32(g.travelled.raw());
    h.write_u32(g.lit as u32);
    for &w in g.pushed.words() {
        h.write_u32(w);
    }
}

fn hash_effect(h: &mut Fnv, e: &Effect) {
    h.write_u32(e.kind as u32 + 1);
    h.write_u32(e.owner as u32);
    h.write_u32(e.class as u32);
    h.write_u32(e.slot as u32);
    h.write_u32(e.age as u32);
    h.write_u32(e.life as u32);
    // Two words to a `u64`, low first: what the halves of
    // the `u128`s these were, so a fight of four hashes as
    // it did.
    for pair in e.struck.words().chunks(2).chain(e.drank.words().chunks(2)) {
        h.write_u64(pair[0] as u64 | (*pair.get(1).unwrap_or(&0) as u64) << 32);
    }
    h.write_i32(e.banked);
    h.write_i32(e.power.raw());
    h.write_i32(e.reach.raw());
    hash_v3(h, &e.pos);
    hash_v3(h, &e.dir);
    hash_v3(h, &e.home);
}

fn hash_player(h: &mut Fnv, p: &Player) {
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
    h.write_u32(p.marks as u32);
    h.write_u32(p.mark_clock as u32);
    h.write_i32(p.thrown_at.raw());
    h.write_u32(p.bound as u32);
    h.write_u32(p.mechanic_held as u32);
    h.write_u32(p.right_held as u32);
    h.write_u32(p.shadow_queued as u32);
    // The fields of 2026-10-09, the Elementalist's and the Blood
    // mage's, hashed only when they hold something -- the convention
    // the creature's `own` words keep -- so a fight with neither class
    // in it hashes as it always did.
    // `earth_held` is the left button's edge, kept for every class
    // but read only by hers; hashed for hers alone.
    let late = [
        (p.class == Class::Elementalist && p.earth_held) as i32,
        p.rise.window as i32,
        p.rise.lifted as i32,
        p.rise.used as i32,
        p.rise.floor.raw(),
        // The Blood mage's charge tier, in the place the jet's paid
        // blood had until the blaster (2026-10-12): nought for every
        // other class, so their hashes are what they were.
        p.tier as i32,
    ];
    if late.iter().any(|v| *v != 0) {
        for v in late {
            h.write_i32(v);
        }
    }
    // The vessel and what a cast owes the floor, only while
    // any holds something -- an empty vessel is nought like the rest.
    let blaster = [
        (p.vessel != NO_VESSEL) as i32 * (p.vessel as i32 + 1),
        p.vessel_tier as i32,
        p.vessel_ready as i32,
        p.paid as i32,
        p.grey_rest as i32,
        p.fuel as i32,
        p.owed as i32,
        // Her territory (2026-10-18): nought for every other class.
        p.on_blood as i32,
        p.heart_meter as i32,
        p.swim as i32,
        p.string as i32,
    ];
    // The stone under her feet, only while there is one (2026-10-18).
    if p.rise.stone != 0 {
        h.write_u32(p.rise.stone as u32);
    }
    if blaster.iter().any(|v| *v != 0) {
        for v in blaster {
            h.write_i32(v);
        }
    }
    // The Dual mage's curse on this body, only while there is one, so
    // a fight she is not in hashes as it did.
    if p.curse != 0 {
        h.write_u32(p.curse as u32);
        h.write_u32(p.curse_clock as u32);
    }
    if p.ball_at != V3::ZERO {
        hash_v3(h, &p.ball_at);
    }
    h.write_u32(p.space_held as u32);
    h.write_u32(p.leap_used as u32);
    // As the byte it was, for every slot a byte could name: a fight
    // of four hashes as it did.
    h.write_u32(if p.blinked == NO_POOL {
        u8::MAX as u32
    } else {
        p.blinked as u32
    });
    h.write_u32(p.bleeding as u32);
    h.write_u32(p.bled_by as u32);
    h.write_u32(p.hauled_in as u32);
    h.write_i32(p.slam.raw());
    h.write_i32(p.slam_fall.raw());
    h.write_i32(p.thrown.raw());
    h.write_u32(p.taken as u32);
    h.write_u32(p.held_by as u32);
    h.write_u32(p.jump_hold as u32);
    h.write_u32(p.air_stall as u32);
    h.write_u32(p.haul as u32);
    hash_v3(h, &p.haul_to);
    h.write_i32(p.haul_speed.raw());
    // The carpets in the byte above, so a fighter who never lays one hashes
    // as before.
    h.write_u32(p.air_stalls as u32 | (p.carpets as u32) << 8);
    // Only once a wall has been touched: a fight that never meets one
    // hashes as it did before there were wall jumps.
    if p.wall_press != 0 {
        h.write_u32(0x3A12);
        h.write_u32(p.wall_press as u32);
    }
    if p.wall.is_some() || p.wall_jumped.is_some() || p.wall_jumps != 0 {
        h.write_u32(0x3A11);
        hash_v3(h, &p.wall.at);
        hash_v3(h, &p.wall.normal);
        h.write_u32(p.wall_since as u32);
        hash_v3(h, &p.wall_jumped.at);
        hash_v3(h, &p.wall_jumped.normal);
        h.write_u32(p.wall_jumps as u32);
    }
    // The same for the slide's state and a ledge: a fight that never slides
    // or hangs hashes as it did before either existed.
    if p.sliding || p.slide_flip != 0 || p.ledge.is_some() || p.ledge_regrab != 0 {
        h.write_u32(0x1ED6);
        h.write_u32(p.sliding as u32);
        h.write_u32(p.slide_flip as u32);
        hash_v3(h, &p.ledge.edge.at);
        hash_v3(h, &p.ledge.edge.normal);
        hash_v3(h, &p.ledge.from);
        h.write_u32(p.ledge.grip as u32);
        h.write_u32(p.ledge.clock as u32);
        h.write_u32(p.ledge.since as u32);
        h.write_u32(p.ledge.fresh as u32);
        h.write_u32(p.ledge_regrab as u32);
    }
    h.write_u32(p.hit_used as u32);
    h.write_u32(p.crouching as u32);
    h.write_u32(p.stride as u32);
    h.write_u32(p.air_frames as u32);
    h.write_u32(p.since_landed as u32);
    h.write_u32(p.parried as u32);
    h.write_u32(p.spent_clicks as u32);
    h.write_u32(p.crouched_for as u32);
    h.write_u32(p.stun_total as u32);
    h.write_u32(p.frozen as u32);
    h.write_u32(p.frozen_bits as u32);
    h.write_i32(p.aloft.raw());
    // Written only once it is something, so a fight that never leaves
    // the floor hashes as it did before there was a bubble.
    if p.clear.raw() != 0 {
        h.write_i32(p.clear.raw());
    }
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
    h.write_u32(monster::mount_hash(p.mount));
    hash_v3(h, &p.local);
    hash_v3(h, &p.grip_vel);
    h.write_i32(p.carry_yaw.raw());
    h.write_u32(p.grip_settle as u32);
    h.write_u32(p.ride_clip as u32);
    h.write_i32(p.beam_reach.raw());
    h.write_i32(p.channelled.raw());
    h.write_u32(p.charging_stone as u32);
    h.write_u32(p.f_held as u32);
    h.write_u32(p.r_held as u32);
    h.write_u32(p.side_b_held as u32);
    h.write_u32(p.breaking as u32);
    // Only when a swing stooped: a fight with nothing short in it
    // hashes as it did before anything could be.
    if p.stoop != Fx::ZERO {
        h.write_i32(p.stoop.raw());
    }
    // Only while a fall could hurt: nothing in the proving ground
    // stands high enough, so its fights hash as they always did.
    if p.fall_over != Fx::ZERO {
        h.write_i32(0xFA11);
        h.write_i32(p.fall_over.raw());
    }
    for f in &p.repeat_lock {
        h.write_u32(*f as u32);
    }
    for f in &p.reactivate_lock {
        h.write_u32(*f as u32);
    }
    hash_mechanic(h, &p.mechanic);
}

fn hash_monster(h: &mut Fnv, slot: usize, m: &crate::monster::Monster) {
    // One for a creature, with its slot and species above the
    // first byte: the Ridgeback in slot zero is still just one.
    h.write_u32(1 | (slot as u32) << 8 | (m.species.0 as u32) << 16);
    // Its temper, only when it has one: a hunt as tuned hashes
    // exactly as it did before there were tempers.
    if m.temper != 0 {
        h.write_u32(0x7E | (m.temper as u32) << 8);
    }
    hash_v3(h, &m.pos);
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
    h.write_u32(m.marks as u32);
    h.write_u32(m.mark_clock as u32);
    // A bleed on a creature, since 2026-10-09: only when there is
    // one, so a hunt nobody bled hashes as it always did.
    if m.bleeding != 0 {
        h.write_u32(m.bleeding as u32);
        h.write_u32(m.bled_by as u32);
    }
    if m.curse != 0 {
        h.write_u32(m.curse as u32);
        h.write_u32(m.curse_clock as u32);
    }
    for part in 0..m.sp().parts.len() {
        h.write_i32(m.part_health(part));
    }
    h.write_u32(m.doing.tag());
    h.write_u32(m.doing.frames_left() as u32);
    h.write_u32(m.doing.attacking().unwrap_or(0) as u32);
    h.write_u32(m.hit_used as u32);
    h.write_u32(m.frozen as u32);
    hash_v3(h, &m.brain.seen);
    hash_v3(h, &m.brain.seen_vel);
    h.write_u32(m.brain.target as u32);
    h.write_u32(m.brain.glance_left as u32);
    h.write_u32(m.brain.think_left as u32);
    h.write_u32(m.brain.last_move as u32);
    h.write_u32(m.brain.repeat_left as u32);
    for lock in &m.brain.cooldown[..m.sp().moves.len()] {
        h.write_u32(*lock as u32);
    }
    h.write_u32(m.brain.rng);
    h.write_u32(m.brain.grace as u32);
    // A species' mark and its words on the body, only when a
    // species has used them: every creature before them hashes
    // as it did.
    if m.brain.aim != [0; 2] {
        h.write_u32(0x3A4C);
        h.write_u32(crate::lore::halves(m.brain.aim[0], m.brain.aim[1]));
    }
    // Getting up or down, and a spray a rock stopped: only when there is
    // one, so a creature that never does either hashes as it did.
    if let crate::monster::Doing::Climb { total, rise, .. } = m.doing {
        h.write_u32(0xC1B | (total as u32) << 16);
        h.write_i32(rise as i32);
    }
    if let Some(stop) = m.brain.stop {
        h.write_u32(0x570B);
        h.write_i32(stop as i32);
    }
    if m.own != [0; 4] {
        h.write_u32(0x0E);
        for w in m.own {
            h.write_i32(w);
        }
    }
}

fn hash_phase(h: &mut Fnv, phase: Phase) {
    match phase {
        Phase::Fighting => h.write_u32(0),
        Phase::RoundOver { winner, left } => {
            h.write_u32(1);
            h.write_u32(winner as u32);
            h.write_u32(left as u32);
        }
    }
}

fn hash_pack(h: &mut Fnv, p: &Pack) {
    h.write_u32(0xC0 | (p.species.0 as u32) << 8);
    if p.temper != 0 {
        h.write_u32(0x7E | (p.temper as u32) << 8);
    }
    for v in [p.mood, p.owner, p.leader, p.boost, p.mustered, p.lost] {
        h.write_u32(v as u32);
    }
    for v in [p.mood_left, p.boost_left, p.glance_left, p.grace] {
        h.write_u32(v as u32);
    }
    for r in &p.rest {
        h.write_u32(*r as u32);
    }
    h.write_u32(p.rng);
    hash_v3(h, &p.home);
    for s in &p.seen {
        hash_v3(h, &s.pos);
        hash_v3(h, &s.vel);
        h.write_u32(s.facing as u32);
        h.write_u32(
            s.alive as u32
                | (s.down as u32) << 1
                | (s.slowed as u32) << 2
                | (s.staggered as u32) << 3,
        );
        h.write_u32(s.ring_places as u32 | (s.ring_base as u32) << 8);
        h.write_u32(s.arc as u32);
    }
    for m in &p.memo {
        h.write_i32(*m);
    }
    for m in &p.per {
        h.write_u32(*m);
    }
}

fn hash_critter(h: &mut Fnv, c: &crate::critter::Critter) {
    hash_v3(h, &c.pos);
    hash_v3(h, &c.vel);
    h.write_u32(c.yaw as u32 | (c.health as u16 as u32) << 16);
    h.write_u32(c.timer as u32 | (c.clock as u32) << 16);
    h.write_u32(u32::from_le_bytes([c.kind, c.state, c.act, c.flags]));
    h.write_u32(
        u32::from_le_bytes([c.role, c.slot, c.target, 0])
            | (monster::mount_hash(c.mount) & 0xFF) << 24,
    );
    if monster::mount_hash(c.mount) > 0xFF {
        h.write_u32(monster::mount_hash(c.mount));
    }
    h.write_u32(c.perch[0] as u16 as u32 | (c.perch[1] as u16 as u32) << 16);
    h.write_u32(c.perch[2] as u16 as u32 | (c.struck_objectives as u32) << 16);
    for seared in c.seared.chunks(4) {
        let mut b = [0; 4];
        b[..seared.len()].copy_from_slice(seared);
        h.write_u32(u32::from_le_bytes(b));
    }
    for &w in c.swung.words() {
        h.write_u32(w);
    }
}

fn hash_lore(h: &mut Fnv, lore: &Lore) {
    h.write_u32(0x10E | (lore.owner.map_or(0xFF, |s| s.0) as u32) << 16);
    for cell in lore.cells() {
        for w in cell {
            h.write_u32(*w);
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
fn step_aloft(p: &mut Player, field: &Field, arena: &Terrain) {
    // How far the feet are above whatever they would land on. **Not `pos.y`**:
    // standing on a platform, on a stone or on the creature's back is
    // standing, and a framing that measured height from the world's origin
    // would swing the view around for somebody who has not left the ground.
    // `grounded` is exactly "my feet are on something", creature included.
    let height = if p.grounded {
        Fx::ZERO
    } else {
        p.pos.y.sub(aim::settle(p.pos, field, arena).y)
    };
    p.aloft = camera::aloft_step(p.aloft, camera::aloft_target(height));
    p.clear = camera::clear_step(p.clear, height);
}

/// Advance the clocks the renderer needs and combat does not.
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
    let moving = p.action.attack_kind().is_some();
    if let Mechanic::Forms { liquid, .. } = &mut p.mechanic {
        liquid.tick(moving);
    }
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
fn settle(p: &mut Player, arena: &Terrain) {
    p.vel.x = p.vel.x.mul(t::settle_decay());
    p.vel.z = p.vel.z.mul(t::settle_decay());
    if !p.grounded {
        p.vel.y = p.vel.y.add(t::gravity().mul(DT));
    }
    p.pos = p.pos.add(p.vel.scale(DT));
    let r = arena.resolve(p.pos, p.vel, p.grounded);
    p.pos = r.pos;
    p.vel = r.vel;
    p.grounded = r.grounded;
}

/// Bodies are solid. Push them apart symmetrically so neither index wins.
///
/// **Except to Thousand Cuts**, which is a run *through* them (exploration
/// 0015, C5): a drawn sword that stopped against the body it was cutting
/// would be the Rush stab again.
///
/// Every pair of the `present` fighters, in seat order; the parked seats are
/// out of the world and push nobody.
fn separate_bodies(players: &mut [Player; MAX_PLAYERS], present: usize) {
    let players = &mut players[..present.min(MAX_PLAYERS)];
    // **Except a body passing through untouchable**: the Dual mage's Flurry
    // dash goes down its own line through whoever is on it (2026-10-12), and
    // stopping at the first body is the dash ending at the thing it was cut
    // through. They are set apart when the dash ends.
    let through = |p: &Player| {
        (p.class == Class::Champion
            && p.action.attack_kind() == Some(moves::champion::THOUSAND_CUTS)
            && rushing(p).is_some())
            || (p.haul > 0 && p.class == Class::DualMage && dual::veiled(p))
    };
    if players.iter().any(through) {
        return;
    }
    let min = t::body_radius().add(t::body_radius());
    for a in 0..players.len() {
        for b in a + 1..players.len() {
            let delta = players[b].pos.sub(players[a].pos);
            let dist = delta.flat_len();
            if dist.raw() == 0 || dist.raw() >= min.raw() {
                continue;
            }
            let push = min.sub(dist).mul(Fx::ratio(1, 2));
            let dir = V3::new(delta.x, Fx::ZERO, delta.z).normalized();
            players[a].pos = players[a].pos.sub(dir.scale(push));
            players[b].pos = players[b].pos.add(dir.scale(push));
        }
    }
}

/// FNV-1a. Chosen because it is trivially portable and has no platform-varying
/// behaviour -- the hash itself must be deterministic or desync detection lies.
pub(crate) struct Fnv(u64);

impl Fnv {
    fn new() -> Fnv {
        Fnv(0xcbf2_9ce4_8422_2325)
    }
    pub(crate) fn write_u64(&mut self, v: u64) {
        self.write_u32(v as u32);
        self.write_u32((v >> 32) as u32);
    }

    pub(crate) fn write_u32(&mut self, v: u32) {
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
    /// Send the shadow to the crosshair, from wherever it is -- and take the
    /// blades with it.
    ///
    /// **Only onto ground she could walk to** (2026-10-10): the send is how
    /// she crosses a gap, with the dash and the jump out of it, and a send
    /// that went up any ledge or over any drop it was pointed at made every
    /// climb in the valley a matter of pointing. See `aim::walk_toward`.
    ///
    /// The second half is the combination the whole kit is built around.
    /// Moving a shadow with a Guillotine open does not cancel the lotus, it
    /// **reactivates** it: the blades stop hanging and start chasing, and since
    /// they track the shadow rather than the ground they drag across the
    /// arena behind it, to wherever it is going now.
    fn order_the_shadow(&mut self, who: usize, to: V3) {
        let stones = stones::gather(&self.players);
        let scene = aim::Scene {
            stones: &stones,
            players: &self.players,
            effects: &self.effects,
            quarry: &self.monsters,
            critters: &self.critters,
            arena: &self.terrain(),
        };
        let footing = aim::walk_toward(self.players[who].pos, to, &scene);
        let Some(order) = shadow::order(&mut self.players[who], footing) else {
            return;
        };
        match order {
            shadow::Order::Refused => {
                // Nothing went out, so nothing is owed: the press can be
                // tried again at once rather than waiting out a lockout on a
                // throw that never happened.
                self.players[who].repeat_lock[SLOT_MECHANIC as usize] = 0;
                self.players[who].reactivate_lock[SLOT_MECHANIC as usize] = 0;
            }
            shadow::Order::Moved => self.drag_the_lotus(who),
            shadow::Order::Sent => {}
        }
    }

    /// The shadow is going somewhere else: an open lotus stops hanging and
    /// chases it there.
    fn drag_the_lotus(&mut self, who: usize) {
        for slot in self.effects.iter_mut().flatten() {
            if slot.kind == EffectKind::GuillotineLotus && slot.owner == who as u8 {
                slot.lotus_send_home();
            }
        }
    }

    /// A shadow's blade landing on the creatures: cut each in reach once for
    /// this part of the effect, and mark it the first time this effect does.
    /// The Whirl, the burst and the thrown blades share it, so all three mark
    /// the same way.
    fn shadow_blades_on_the_creatures(
        &mut self,
        effect: &mut Effect,
        part: usize,
        at: V3,
        radius: Fx,
    ) {
        for slot in 0..MAX_MONSTERS {
            let victim = quarry_victim(slot);
            let dealt = self.gore_for_a_share(slot, effect, part, at, radius, Fx::ONE);
            if dealt > 0 {
                effect.take_hit(part, victim);
                if marks_once(effect, victim) {
                    if let Some(beast) = self.monsters[slot].as_mut() {
                        beast.mark();
                    }
                }
            }
        }
    }

    /// What the Reaver's moves do on the frame they come out, besides their
    /// volume (2026-10-10). The shadow's two in the air and off the floor put
    /// it somewhere and set it cutting; the Tumble throws its fan; the
    /// Plunge's first active frame is the one her feet arrive on, and the
    /// ring runs out from there.
    fn reaver_comes_out(&mut self, who: usize, kind: u8) {
        use moves::reaver as r;
        let p = self.players[who];
        let m = moves::get(p.class, kind);
        match kind {
            r::WHIRL | r::RISING_WHIRL => {
                let was_out = shadow::of(&p).is_some_and(|s| s.is_out());
                // **The crosshair's point is the whirl's middle**, as it is
                // the middle of what stands there for every skillshot
                // (`aim::standing_middle`): the shadow's feet go half a body
                // under it, and never under the floor. With its feet on the
                // point, a level look hung the whirl over everybody's heads.
                //
                // **And it stops at the first body on its way** (2026-10-12):
                // the crosshair's ray goes through bodies to the place behind
                // them, so a whirl aimed at somebody -- or at a sac on the
                // Broodmother -- would otherwise spin on the floor past them.
                // What the throw runs into is `aim::first_along`'s.
                let to = {
                    let stones = stones::gather(&self.players);
                    let scene = aim::Scene {
                        stones: &stones,
                        players: &self.players,
                        effects: &self.effects,
                        quarry: &self.monsters,
                        critters: &self.critters,
                        arena: &self.terrain(),
                    };
                    let path = aim::Path {
                        from: p.aim_path.from,
                        to: p.aim_at(),
                    };
                    let targets = aim::Targets::none()
                        .fighters(self.pvp())
                        .spare(self.allies(who))
                        .quarry(true);
                    match aim::first_along(path, Fx::ZERO, who as u8, &scene, targets) {
                        Some(met) => path.from.add(path.dir().scale(met.dist())),
                        None => path.to,
                    }
                };
                let floor = self.terrain().ground_under(to);
                let feet = to.y.sub(t::body_height().mul(Fx::ratio(1, 2))).max(floor);
                let to = V3::new(to.x, feet.min(to.y), to.z);
                shadow::throw(&mut self.players[who], to, t::shadow_return());
                if was_out {
                    self.drag_the_lotus(who);
                }
                // **One set of blades** (2026-10-12): pressed again, the
                // whirl already spinning goes with the shadow and starts its
                // clock over, rather than a second set joining it. A fresh
                // spin is a fresh attack, so it may mark again.
                let at = self.players[who].mechanic.placed().unwrap_or(p.pos);
                let spinning = self
                    .effects
                    .iter_mut()
                    .flatten()
                    .find(|e| e.kind == EffectKind::ShadowWhirl && e.owner == who as u8);
                match spinning {
                    Some(e) => {
                        e.age = 0;
                        e.slot = kind;
                        e.pos = at;
                        e.banked = 0;
                        e.forget_hits();
                    }
                    None => spawn_effect(
                        &mut self.effects,
                        Effect::cast(
                            EffectKind::ShadowWhirl,
                            who as u8,
                            p.class,
                            kind,
                            at,
                            V3::ZERO,
                            m.reach,
                        ),
                    ),
                }
            }
            // The Tumble's fan, down the crosshair's line, and the Fan's,
            // level across it.
            r::TUMBLE | r::FAN => {
                self.throw_blades(who, kind, p.aim_path.from, p.aim_path.dir(), m.reach);
            }
            // **The Shuriken**: hers to the crosshair's point, and the
            // shadow's to the same point from wherever it is standing.
            r::SHURIKEN => {
                let to = p.aim_at();
                let from_her = p.aim_path.from;
                let from_it = p.mechanic.placed().map(|at| {
                    at.add(V3::new(
                        Fx::ZERO,
                        t::body_height().mul(Fx::ratio(1, 2)),
                        Fx::ZERO,
                    ))
                });
                for from in [Some(from_her), from_it].into_iter().flatten() {
                    let span = to.sub(from);
                    let far = span.len();
                    if far.raw() > 0 {
                        self.throw_blades(who, kind, from, span.normalized(), far);
                    }
                }
            }
            // **The Reave's dash stops where it strikes.** The step's speed
            // is a dash's -- thirty-odd metres a second for a few frames --
            // and the follow-through every other step keeps would slide her
            // on past the sweet spot, or launch her off a ledge at that speed.
            r::REAVE => {
                let q = &mut self.players[who];
                q.vel = V3::new(Fx::ZERO, q.vel.y, Fx::ZERO);
            }
            r::PLUNGE => spawn_effect(
                &mut self.effects,
                Effect::cast(
                    EffectKind::ShadowRing,
                    who as u8,
                    p.class,
                    kind,
                    p.pos,
                    V3::ZERO,
                    t::shadow_ring_reach(),
                ),
            ),
            _ => {}
        }
    }

    /// Thrown blades from `from` along `dir`, out to `reach` at
    /// `tuning::thrown_speed`: a Shuriken, the Tumble's fan or the Fan's.
    /// How many and how they spread is the move's (`moves::reaver::blades`,
    /// `Effect::fan_dir`).
    fn throw_blades(&mut self, who: usize, kind: u8, from: V3, dir: V3, reach: Fx) {
        let p = self.players[who];
        let mut born = Effect::cast(
            EffectKind::ShadowFan,
            who as u8,
            p.class,
            kind,
            from,
            dir,
            reach,
        );
        let step = t::thrown_speed().mul(DT);
        born.life = if step.raw() > 0 {
            reach.div(step).to_int().clamp(1, u16::MAX as i32) as u16
        } else {
            1
        };
        spawn_effect(&mut self.effects, born);
    }

    /// The shadow is back with her because she went and got it -- the dash,
    /// standing on it, Gallows' blink: **the burst** (2026-10-12), blades
    /// thrown round her as it rejoins her. Its coming home on its own clock is
    /// not a collection and bursts nothing.
    fn burst_the_collected(&mut self) {
        for who in 0..MAX_PLAYERS {
            if !shadow::take_collected(&mut self.players[who]) {
                continue;
            }
            let p = self.players[who];
            spawn_effect(
                &mut self.effects,
                Effect::cast(
                    EffectKind::ShadowBurst,
                    who as u8,
                    p.class,
                    moves::reaver::SEND,
                    p.pos,
                    V3::ZERO,
                    t::burst_radius(),
                ),
            );
        }
    }

    /// What the second body does to the other one, this frame.
    fn step_shadows(&mut self) {
        for owner in 0..MAX_PLAYERS {
            self.aim_the_copy(owner);
            self.echo_strikes(owner);
            self.recall_cuts(owner);
        }
    }

    /// Out on the field, the shadow turns to whoever is in reach while its copy
    /// winds up -- and keeps her yaw if nobody is. The decision is
    /// `aim::shadow_faces`'s; this only hands it the scene.
    ///
    /// The scene is built only on the frames a copy is winding up out there,
    /// which is a handful of frames per swing for one class. Every other frame
    /// for every other fighter this is a match on the mechanic and nothing
    /// more.
    fn aim_the_copy(&mut self, owner: usize) {
        let Some((at, reach)) = shadow::copy_needs_a_target(&self.players[owner]) else {
            return;
        };
        let stones = stones::gather(&self.players);
        let scene = aim::Scene {
            stones: &stones,
            players: &self.players,
            effects: &self.effects,
            quarry: &self.monsters,
            critters: &self.critters,
            arena: &self.terrain(),
        };
        let facing = aim::shadow_faces(at, owner, reach, !self.hunting(), &scene)
            .unwrap_or(self.players[owner].facing);
        shadow::turn_to(&mut self.players[owner], facing);
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
        // **Where it is standing decides what the copy is worth and whether it
        // counts.** Out on the field it deals the full share and marks what it
        // hits -- that is the tally. At her heel it deals half that and marks
        // nothing: it is at her shoulder, and there is nothing being set up.
        let out = shadow::of(&self.players[owner]).is_some_and(|s| s.is_out());
        let share = if out {
            t::shadow_echo()
        } else {
            t::shadow_echo_attending()
        };
        let landed = if self.hunting() {
            self.echo_gores_the_creature(owner, &ghost, share, out)
        } else if self.pvp() {
            self.echo_cuts_the_other_fighter(owner, &ghost, share, out)
        } else {
            false
        };
        if landed {
            shadow::echo_landed(&mut self.players[owner]);
        }
    }

    fn echo_cuts_the_other_fighter(
        &mut self,
        owner: usize,
        ghost: &Player,
        share: Fx,
        marks: bool,
    ) -> bool {
        // Every other fighter its swing reaches, as hers would.
        let mut landed = false;
        for target in 0..self.present() {
            let defender = self.players[target];
            if !self.foes(owner, target) || defender.health <= 0 {
                continue;
            }
            let Some(mut hit) = resolve_hit(ghost, &defender, owner as u8) else {
                continue;
            };
            hit.damage = Fx::from_int(hit.damage).mul(share).to_int();
            hit.parried = false;
            apply_hit(&mut self.players[target], hit);
            // A mark for a hit that *landed*. Blocked, the copy did what a
            // blocked swing does and nothing more; the tally counts what got
            // through.
            if marks && !hit.blocked {
                shadow::mark(&mut self.players[target]);
            }
            landed = true;
        }
        landed
    }

    fn echo_gores_the_creature(
        &mut self,
        owner: usize,
        ghost: &Player,
        share: Fx,
        marks: bool,
    ) -> bool {
        let (Some(box_out), Some(kind)) = (hitbox(ghost), ghost.action.attack_kind()) else {
            return false;
        };
        if box_out.spent {
            return false;
        }
        // The first creature the copy reaches, in slot order: one swing, one
        // body, the same as hers.
        for place in self.monsters.iter_mut() {
            let Some(beast) = place.as_mut().filter(|b| b.alive()) else {
                continue;
            };
            let Some(part) = part_under(beast, ghost, &box_out) else {
                continue;
            };
            let m = moves::get(ghost.class, kind);
            let raw = Fx::from_int(m.damage)
                .mul(preying(ghost.class, beast.disabled()))
                .mul(share)
                .to_int();
            // **A copy is guarded against from where the copy stands** -- so
            // one thrown from a shadow behind a guard lands (mantis.md §4).
            let (_, guarded) = beast.take_blow(
                part,
                raw,
                &monster::Blow {
                    from: ghost.pos,
                    unblockable: m.unblockable,
                    who: owner as u8,
                    class: ghost.class,
                    kind,
                },
            );
            if marks && !guarded.turned() {
                beast.mark();
            }
            return true;
        }
        false
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
        if self.hunting() {
            for slot in 0..MAX_MONSTERS {
                let Some(mut beast) = self.monsters[slot] else {
                    continue;
                };
                let victim = quarry_victim(slot);
                if beast.alive() && !shadow::already_cut(ghost, victim) {
                    if let Some(part) = beast.part_struck(ghost.pos, reach, t::body_height()) {
                        let raw = Fx::from_int(m.damage)
                            .mul(preying(Class::ShadowReaver, beast.disabled()))
                            .to_int();
                        let (_, guarded) = beast.take_blow(
                            part,
                            raw,
                            &monster::Blow {
                                from: ghost.pos,
                                unblockable: m.unblockable,
                                who: owner as u8,
                                class: Class::ShadowReaver,
                                kind: SLOT_MECHANIC,
                            },
                        );
                        if !guarded.turned() {
                            beast.mark();
                            // The recall's slow, offered the same way
                            // everything else is. It is half the reason to
                            // recall through something.
                            beast.take_control(monster::Control::slowing(
                                t::slow_frames(),
                                t::shadow_recall_slow(),
                            ));
                        }
                        self.monsters[slot] = Some(beast);
                        shadow::mark_cut(&mut self.players[owner], victim);
                    }
                }
            }
            return;
        }

        // Every other fighter it passes through, each once.
        for target in 0..self.present() {
            if target == owner {
                continue;
            }
            if shadow::already_cut(ghost, target) {
                continue;
            }
            let victim = self.players[target];
            if victim.health <= 0 || victim.untouchable() {
                continue;
            }
            let apart = V3::new(
                victim.pos.x.sub(ghost.pos.x),
                Fx::ZERO,
                victim.pos.z.sub(ghost.pos.z),
            );
            if apart.flat_len().raw() > reach.raw() {
                continue;
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
            // One mark for the way home, the way it cuts once. Not if he blocked
            // it: the tally counts what got through.
            if !guarding {
                shadow::mark(&mut self.players[target]);
            }
            shadow::mark_cut(&mut self.players[owner], target);
        }
    }
}

/// Spend the marks on `defender` if this blow is the Reaver's own, and say
/// whether it was a full tally -- the one that also staggers.
///
/// **Only her finishers cash** (2026-10-12): the Reave, the Plunge and
/// Gallows -- `moves::reaver::cashes`. Every one of them landing on a marked
/// target multiplies by the marks and clears them; the shadow's blades put
/// them on, the finishers take them off. Until 2026-10-12 any hit of hers
/// cashed, so a poke spent the tally a poke had built and going in was
/// worth nothing extra; from play, "the pattern should be low damage into a
/// high, super impactful burst with the lclick moves". The copies are not her body -- they go through
/// `echo_cuts_the_other_fighter` and mark rather than spend, or the tally
/// could never climb past one. **Blocked or parried, nothing is spent**: it
/// was not a hit, and the tally stays on him. That is the counterplay the pips
/// exist to make possible. See `docs/design/shadow-reaver-v2.md`.
fn cash_the_tally(
    players: &mut [Player; MAX_PLAYERS],
    attacker: usize,
    defender: usize,
    hit: &mut Hit,
) -> bool {
    let marks = players[defender].marks;
    let finisher = players[attacker]
        .action
        .attack_kind()
        .is_some_and(moves::reaver::cashes);
    if players[attacker].class != Class::ShadowReaver
        || !finisher
        || hit.blocked
        || hit.parried
        || marks == 0
    {
        return false;
    }
    hit.damage = Fx::from_int(hit.damage)
        .mul(shadow::cash_multiple(marks))
        .to_int();
    players[defender].marks = 0;
    players[defender].mark_clock = 0;
    shadow::full_tally(marks)
}

/// **A shadow-blade attack marks each body once** (2026-10-12): the first
/// time an effect of the Reaver's lands on a victim, and never again for that
/// effect -- a throw, a spin, a burst. Kept in the effect's `banked`, a bit a
/// victim. Says whether this was that first time.
fn marks_once(effect: &mut Effect, victim: usize) -> bool {
    let bit = 1i32 << victim;
    let first = effect.banked & bit == 0;
    effect.banked |= bit;
    first
}

/// **The Reaping** (2026-10-19): put the picture of a cash-in at a full tally
/// in the world (`EffectKind::Reaping`). `met` is where the blow met what it
/// cut -- a body's middle, or the place on a creature's part -- `floor` the
/// floor under that, and `thick` how far the thing cut reaches out from
/// `met` across the floor (a body's radius, a leg's, half a back):
/// what the picture needs to stand its copies of her on the floor round a
/// fighter, a leg or a whole animal, as big as it takes to reach the place.
/// It faces the way the blow went, from her to it. Nothing but a picture; in
/// the snapshot because its clock must survive a rollback like any
/// animation's.
fn reap(
    effects: &mut [Option<Effect>; MAX_EFFECTS],
    attacker: &Player,
    who: usize,
    met: V3,
    floor: V3,
    thick: Fx,
) {
    let away = V3::new(
        met.x.sub(attacker.pos.x),
        Fx::ZERO,
        met.z.sub(attacker.pos.z),
    );
    let facing = V3::new(attacker.facing.x, Fx::ZERO, attacker.facing.z);
    let dir = if away.flat_len().raw() > 0 {
        away.normalized()
    } else {
        facing.normalized()
    };
    let mut reaping = Effect::cast(
        EffectKind::Reaping,
        who as u8,
        attacker.class,
        attacker.action.attack_kind().unwrap_or(0),
        met,
        dir,
        thick,
    );
    reaping.home = floor;
    spawn_effect(effects, reaping);
}

/// A cash-in at a full tally is a hard stop: the stagger replaces the swing's
/// own hitstun, and only if it is longer.
fn stagger_the_cashed(p: &mut Player) {
    if p.health <= 0 {
        return;
    }
    let left = t::cash_stagger().max(p.action.frames_left());
    p.action = Action::Stagger { left };
    p.stun_total = left;
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
        self.step_plagues();
        self.light_the_fuses();
        self.step_hearts();
        self.step_thralls();
        self.slide_the_stains();
        self.step_ground();
        for i in 0..MAX_EFFECTS {
            let Some(mut effect) = self.effects[i] else {
                continue;
            };
            let turning = !effect.returning();
            let going_out = !effect.lotus_coming_back();
            // A thrown thing on its first frame: no critter has been cut by
            // this one yet, whatever the last of its kind did.
            if effect.age == 0 && effect.kind.once_a_pass() {
                for c in self.critters.iter_mut() {
                    c.sear(effect.owner, effect.slot, false);
                }
            }
            // Held at the top of its counter: a heart, or a thrall on her
            // ground, can stand longer than it counts.
            effect.age = effect.age.saturating_add(1);
            // **Her moves claim the floor they touch** (2026-10-18): what she
            // puts on the floor lays her ground there, on its first frame.
            if effect.age == 1 && effect.touches_the_floor() {
                self.lay_touch(effect.owner as usize, effect.pos, effect.field_radius());
            }
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
            // And the one that stays under its caster's feet: the Downdraft
            // is a column she carries down with her.
            if effect.kind.follows_the_caster() {
                if let Some(owner) = self.players.get(effect.owner as usize) {
                    effect.pos = owner.pos;
                }
            }
            // The Whirl is the shadow's own blades: once it is back with her
            // -- collected, or on its way home -- they are over, rather than
            // spinning at her heel.
            if effect.kind == EffectKind::ShadowWhirl
                && !self
                    .players
                    .get(effect.owner as usize)
                    .and_then(shadow::of)
                    .is_some_and(|s| s.is_out() && !s.is_returning())
            {
                effect.age = effect.life;
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
            // A thrall standing on her ground does not (`step_thralls`), and
            // a heart never does: it stands until it is beaten down or a
            // newer one takes its place (2026-10-19).
            if effect.is_a_pool() || (effect.is_a_thrall() && !effect.struck.has(ON_GROUND)) {
                effect.banked -= effect.pool_drained_this_frame();
            }
            let expired = if effect.kind == EffectKind::FireTornado {
                let flying = effect.age.saturating_sub(effect.banked as u16);
                flying >= t::tornado_travel_life() || !self.terrain().inside(effect.tornado_pos())
            } else if effect.is_a_pool() || effect.is_a_thrall() || effect.is_a_heart() {
                effect.banked <= 0
            } else {
                effect.age >= effect.life
            };
            if expired {
                self.effects[i] = None;
                // **A stain bleeds into her ground as it fades** (2026-10-18):
                // drained away, rather than drunk or set off, it leaves a
                // patch of her ground where it stood.
                if effect.is_a_pool() && (effect.owner as usize) < MAX_PLAYERS {
                    self.lay_ground(
                        effect.owner,
                        effect.pos,
                        effect.stain_ground(),
                        t::ground_stain_lasts(),
                    );
                }
                // The Thorn lash at the end of its flight, hit or miss.
                self.land_the_lash(&effect);
                self.pay_out(&effect);
                continue;
            }
            self.apply_effect(&mut effect);
            // The bolt is spent by what it just did -- it reached a body --
            // and goes now rather than being drawn one more frame at the far
            // end of a flight it never made. Nothing else expires here: a
            // tornado, for one, is older than its life by design.
            let spent = (matches!(effect.kind, EffectKind::Haemorrhage | EffectKind::Nail)
                && effect.age >= effect.life)
                || (effect.kind == EffectKind::AirBall && effect.reach.raw() <= 0);
            if spent {
                self.land_the_lash(&effect);
            }
            self.effects[i] = (!spent).then_some(effect);
        }
        self.step_bleeds();
        self.bleed_the_creatures();
        self.step_spellbook();
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
            // The Reaver's marks fade one at a time, so a target who gets away
            // from the shadow slowly cleans himself. Here beside the slow's
            // tail because it is the same kind of thing: a count on the victim
            // that whatever set it refreshes, and that runs down otherwise.
            shadow::fade_mark(p);
        }
    }

    /// A tick of a bleed `owner` opened: `tuning::bleed_damage`, and harder
    /// while she stands on her ground (`worth_of`, 2026-10-19) -- her
    /// territory's strength is in everything she deals, not only what she
    /// throws from it.
    fn bleed_tick_of(&self, owner: usize) -> i32 {
        Fx::from_int(t::bleed_damage())
            .mul(worth_of(&self.players[owner], None))
            .to_int()
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
            let dealt = self.players[i].wound(self.bleed_tick_of(owner));
            let class = self.players[owner].class;
            self.spill_under(owner as u8, class, moves::blood::THORN_LASH, &p, dealt);
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
        if effect.kind == EffectKind::Grasp {
            self.haul_to_the_grasp(effect);
            return;
        }
        if effect.kind == EffectKind::Quake {
            self.erupt_the_quake(effect);
            return;
        }
        if effect.kind == EffectKind::Nail
            && effect.class == Class::BloodMage
            && effect.slot == moves::blood::STAKE
        {
            self.land_the_stake(effect);
            return;
        }
        if effect.kind != EffectKind::Bloodletter || effect.banked <= 0 {
            return;
        }
        let owed = effect.leeched(effect.banked);
        if let Some(caster) = self.players.get_mut(effect.owner as usize) {
            caster.heal(owed);
        }
    }

    /// **A Grasp that closed on scenery pulls her to it** -- the Blood mage's
    /// movement, behind `tuning::grasp_hauls`.
    ///
    /// The ability already exists to close a gap: four arms converge on a point
    /// and whatever they catch is hauled back to her. This is the same sentence
    /// with the subject swapped. Catch a person and it works as it always has;
    /// catch a wall, a structure or the creature and the arms have something to
    /// pull against and nothing to pull, so what moves is her.
    ///
    /// Asked on the frame the arms are done, which is the only frame it can be
    /// asked on: "all four and nobody in them" is not knowable until the fourth
    /// has landed or failed to, and the arms land a frame apart.
    ///
    /// **Whiffing it is still whiffing it.** The health went on the press and
    /// none of it comes back -- nothing was cut -- so using the Grasp as a
    /// grappling hook costs her the full price of a missed cast every time. That
    /// is the whole reason it is allowed to be this strong: it is movement she
    /// pays for out of the resource she is made of.
    fn haul_to_the_grasp(&mut self, effect: &Effect) {
        if !t::grasp_hauls() {
            return;
        }
        let owner = effect.owner as usize;
        // Somebody in the arms means there was something to pull. The arms do
        // not do both jobs at once.
        if (0..MAX_PLAYERS).any(|i| effect.parts_landed(i, GRASP_ARMS) == GRASP_ARMS) {
            return;
        }
        let Some(caster) = self.players.get(owner) else {
            return;
        };
        if caster.health <= 0 {
            return;
        }
        // Where the arms met. `pos`, `dir` and `reach` were fixed on the frame
        // she let go, so this is the point she was promised rather than a
        // second guess at it.
        let met = effect.pos.add(effect.dir.scale(effect.reach));
        let stones = stones::gather(&self.players);
        let scene = aim::Scene {
            stones: &stones,
            players: &self.players,
            effects: &self.effects,
            quarry: &self.monsters,
            critters: &self.critters,
            arena: &self.terrain(),
        };
        // **Is there anything to pull on?** Asked along the arms' own path with
        // the shared question `aim::first_along` already answers for every
        // thrown thing in the game -- see `aim::Contact::is_an_anchor`. A
        // fighter is not an anchor and neither is a fire: one of them moves and
        // the other is not there.
        let path = aim::Path {
            from: effect.pos,
            to: met,
        };
        let found = aim::first_along(
            path,
            t::grasp_arm_radius(),
            effect.owner,
            &scene,
            aim::Targets::none()
                .terrain()
                .stones()
                .quarry(self.hunting()),
        );
        let Some(anchor) = found.filter(|c| c.is_an_anchor()) else {
            return;
        };
        // To the anchor rather than to the point the arms converged on, which
        // may be a long way behind it: the Grasp's reach is the hold and the
        // hold does not shorten itself against the scenery (see the kit
        // document), so a full wind-up at a near wall converges metres inside
        // it. She arrives at the surface, not inside it.
        // Dropped on to whatever is underneath it. The ray was cast from her
        // chest so that it meets the same geometry an ability would, and where
        // she *stands* is the floor -- or the platform top -- under the far end
        // of it.
        let stop = aim::settle(
            path.at(anchor.dist().sub(t::body_radius())),
            &stones,
            &self.terrain(),
        );
        let speed = t::grasp_haul_speed();
        // Long enough to cover the whole of the Grasp's reach at that speed,
        // with a frame in hand. Worked out rather than written down, so
        // retuning either the reach or the speed cannot leave a pull that stops
        // in mid-air.
        let frames = effect.reach.div(speed.mul(DT)).to_int().clamp(1, 240) as u16 + 1;
        begin_haul(&mut self.players[owner], stop, speed, frames);
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
            // **The Champion's full charges** (exploration 0015, C5).
            //
            // The crescent: one wide blade on its line, each body once.
            EffectKind::Crescent => {
                let at = effect.head();
                let radius = effect.field_radius();
                for i in 0..MAX_PLAYERS {
                    if !self.effects_reach(i, effect.owner)
                        || effect.already_hit(0, i)
                        || !self.inside(i, at, radius)
                    {
                        continue;
                    }
                    effect.take_hit(0, i);
                    let dealt = self.cut(i, effect, at, effect.damage());
                    self.feed_the_thrower(effect, dealt);
                }
                let dealt = self.gore_the_creature(effect, 0, at, radius);
                self.feed_the_thrower(effect, dealt);
            }
            // The ring: the front of it, on the floor, each body once, and
            // the move's own launch throws them up.
            EffectKind::Shockwave => {
                let r = effect.ring_radius();
                let band = t::body_radius().add(t::disc_thickness());
                for i in 0..MAX_PLAYERS {
                    let q = self.players[i];
                    let across = V3::new(
                        q.pos.x.sub(effect.pos.x),
                        Fx::ZERO,
                        q.pos.z.sub(effect.pos.z),
                    )
                    .flat_len();
                    let low = q.pos.y.sub(effect.pos.y).raw() < t::body_height().raw();
                    if !self.effects_reach(i, effect.owner)
                        || effect.already_hit(0, i)
                        || !low
                        || across.sub(r).abs().raw() > band.raw()
                        || q.action.invulnerable()
                    {
                        continue;
                    }
                    effect.take_hit(0, i);
                    let dealt = self.cut(i, effect, effect.pos, effect.damage());
                    self.feed_the_thrower(effect, dealt);
                }
                // The creature and the small bodies inside its live radius,
                // once each: the Plunge's ring's rule.
                let reach = r.add(band);
                for slot in 0..MAX_MONSTERS {
                    self.gore_for_a_share(slot, effect, 0, effect.pos, reach, Fx::ONE);
                }
                self.gore_critters_by(effect, EffectReach::Column(effect.pos, reach), Fx::ONE);
            }
            // The cuts: nothing for the beat, then each lands along the whole
            // line in turn, and the last throws up whoever is still in it.
            EffectKind::Cuts => {
                if let Some(n) = effect.cut_now() {
                    let (from, to) = (effect.pos, effect.cuts_end());
                    let radius = effect.field_radius();
                    let damage = effect.cut_damage(n);
                    let last = Effect::last_cut(n);
                    for i in 0..MAX_PLAYERS {
                        if !self.effects_reach(i, effect.owner)
                            || effect.already_hit(n, i)
                            || !self.on_the_line(i, from, to, radius)
                        {
                            continue;
                        }
                        effect.take_hit(n, i);
                        let dealt = self.cut(i, effect, from, damage);
                        self.feed_the_thrower(effect, dealt);
                        if last && dealt > 0 {
                            let q = &mut self.players[i];
                            q.vel.y = t::cuts_launch();
                            q.grounded = false;
                        }
                    }
                    let share = Fx::ratio(damage, effect.damage().max(1));
                    for k in 0..=4 {
                        let at = crate::math::lerp3(from, to, Fx::ratio(k, 4));
                        for slot in 0..MAX_MONSTERS {
                            self.gore_for_a_share(slot, effect, n, at, radius, share);
                        }
                        self.gore_critters(effect, at, radius, share);
                    }
                }
            }
            // The javelin: in flight it pierces and pins, each body once;
            // at rest it waits for him, and he tears it out.
            EffectKind::Javelin => {
                let at = effect.head();
                let radius = effect.field_radius();
                if !effect.at_rest() {
                    for i in 0..MAX_PLAYERS {
                        if !self.effects_reach(i, effect.owner)
                            || effect.already_hit(0, i)
                            || !self.inside(i, at, radius)
                        {
                            continue;
                        }
                        effect.take_hit(0, i);
                        let dealt = self.cut(i, effect, at, effect.damage());
                        self.feed_the_thrower(effect, dealt);
                        if dealt > 0 {
                            // It stops in them: pinned where it struck, and
                            // that is where he is coming to.
                            self.players[i].seized(NOBODY, effect.source().grabs, 0);
                            effect.reach = effect.flown();
                        }
                    }
                    if !effect.at_rest() && self.gore_the_creature(effect, 0, at, radius) > 0 {
                        // Stuck in the animal.
                        effect.reach = effect.flown();
                    }
                } else {
                    self.tear_out(effect, at);
                }
            }
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
                // A field of spikes (the third tier) comes up again every
                // `tuning::spike_wave_every`, and each wave can catch anybody
                // the last one did.
                let every = t::spike_wave_every().max(1);
                let wave =
                    effect.banked == SPIKE_FIELD && effect.age > 1 && (effect.age - 1) % every == 0;
                if wave {
                    effect.forget_hits();
                }
                let spreading = effect.spreads() && effect.age <= t::erupt_spread();
                if effect.erupted() && (effect.age == 1 || wave || spreading) {
                    let volume = effect.spike_volume();
                    let m = effect.source();
                    // **An eruption sets off what its edge reaches**, as it
                    // spreads (2026-10-19): each stain it touches lights.
                    if spreading && (effect.owner as usize) < MAX_PLAYERS {
                        // What it sets off past its edge grows with it, so
                        // each blast reaches the next as it spreads rather
                        // than on its first frame.
                        let spread = t::erupt_spread().max(1) as i32;
                        let grown = Fx::ratio((effect.age as i32).clamp(1, spread), spread);
                        let past = t::erupt_chain().mul(grown).sub(t::erupt_chain());
                        self.light_round(
                            effect.owner as usize,
                            effect.pos,
                            volume.radius.add(past),
                        );
                    }
                    let mut landed = 0;
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
                        let blow = Fx::from_int(m.damage).mul(spike_share(effect)).to_int();
                        let dealt = self.cut(i, effect, effect.pos, blow);
                        self.players[i].slow(t::slow_frames(), t::spike_slow());
                        let owed = effect.leeched(dealt);
                        self.players[effect.owner as usize].heal(owed);
                        landed += dealt;
                    }
                    let share = spike_share(effect);
                    let each = self.gore_each(effect, 0, effect.pos, volume.radius, share);
                    // **A spreading eruption hits a creature once** (2026-10-19):
                    // a spike is not a pass the creature path keeps books on,
                    // which did not matter while it hit only on its first
                    // frame. A spread asks on every frame of it, and without
                    // this a Ridgeback took the eruption on each of them.
                    for (slot, &got) in each.iter().enumerate() {
                        if got > 0 {
                            effect.take_hit(0, quarry_victim(slot));
                        }
                    }
                    // The small bodies have no such book on an effect but
                    // their own searing, so a spread meets them once, at its
                    // full size.
                    let critters_now = if effect.spreads() {
                        effect.age == t::erupt_spread()
                    } else {
                        effect.age == 1 || wave
                    };
                    let dealt = each.iter().sum::<i32>()
                        + if critters_now {
                            self.gore_critters(effect, effect.pos, volume.radius, share)
                        } else {
                            0
                        };
                    let owed = effect.leeched(dealt);
                    self.players[effect.owner as usize].heal(owed);
                    landed += dealt;
                    if effect.age == 1 && landed > 0 {
                        self.erupt_from_a_hit(effect, landed);
                    }
                }
            }

            // **The Hemoplague**: on its tick, everyone in the cloud is hurt
            // -- not hit: no stun and no shove, so it holds nobody -- and
            // what it spills goes to one stain at its centre.
            EffectKind::Plague => {
                if effect.age % t::plague_tick().max(1) != 0 {
                    return;
                }
                self.step_the_plague(effect);
            }

            // **The Black vines** and **the Crimson cascade**: each frame
            // decides for itself what lands on it.
            EffectKind::Vines => self.step_the_vines(effect),
            // The Rising reap's sweep, which is only to look at.
            EffectKind::Cascade => {}
            // Fissure's surge. See `step_the_surge`.
            EffectKind::Surge => self.step_the_surge(effect),

            // A thrall and a heart are stepped in passes of their own
            // (`step_thralls`, `step_hearts`).
            EffectKind::Thrall | EffectKind::Heart => {}
            // The Reaping is only to look at: the blow that cashed the tally
            // already did everything it does.
            EffectKind::Reaping => {}

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
                    // Coming home, a blade is worth a share of what it was
                    // going out -- to a fighter and to the creature alike.
                    let share = if coming_back {
                        t::lotus_return_damage()
                    } else {
                        Fx::ONE
                    };
                    for i in 0..MAX_PLAYERS {
                        // Only the first few blades of a pass to reach a body
                        // cut it; the rest go through. See
                        // `tuning::lotus_blades_a_pass`.
                        let landed = (0..LOTUS_BLADES)
                            .filter(|&b| effect.already_hit(b, i))
                            .count();
                        if !self.effects_reach(i, effect.owner)
                            || effect.already_hit(blade, i)
                            || landed >= t::lotus_blades_a_pass()
                            || !self.sliced(i, was, at, radius, half_thick)
                        {
                            continue;
                        }
                        // The first blade of this pass to reach somebody
                        // marks them, and the others do not: once per pass,
                        // so a flower out and home is two marks rather than a
                        // full tally on its own. The mask is cleared at the
                        // turn, which is what makes it once per *pass*.
                        let first_this_pass = landed == 0;
                        effect.take_hit(blade, i);
                        let blow = Fx::from_int(effect.source().damage).mul(share).to_int();
                        let dealt = self.cut(i, effect, at, blow);
                        self.players[i].slow(t::slow_frames(), t::lotus_slow());
                        if first_this_pass && dealt > 0 {
                            shadow::mark(&mut self.players[i]);
                        }
                    }
                    // The same count for each creature, which is wide enough
                    // that every blade reaches it: without it, twelve hits a
                    // pass.
                    for slot in 0..MAX_MONSTERS {
                        let landed = (0..LOTUS_BLADES)
                            .filter(|&b| effect.already_hit(b, quarry_victim(slot)))
                            .count();
                        if landed >= t::lotus_blades_a_pass() {
                            continue;
                        }
                        let first_this_pass = landed == 0;
                        if self.gore_for_a_share(slot, effect, blade, at, radius, share) > 0
                            && first_this_pass
                        {
                            if let Some(beast) = self.monsters[slot].as_mut() {
                                beast.mark();
                            }
                        }
                    }
                    // The small bodies: each blade's own slab, and a critter is
                    // cut once by the whole flower (`Critter::seared`), so
                    // twelve blades are not twelve cuts.
                    self.gore_critters_by(
                        effect,
                        EffectReach::Slab(was, at, radius, half_thick),
                        share,
                    );
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
                let each = self.gore_each(effect, 0, effect.pos, radius, Fx::ONE);
                for (slot, dealt) in each.into_iter().enumerate() {
                    if dealt > 0 {
                        let owed = effect.leeched(dealt);
                        self.players[effect.owner as usize].heal(owed);
                        effect.take_hit(0, quarry_victim(slot));
                    }
                }
                // And the small bodies in it, on the burst's first frame only:
                // a burst is one pass, and they have no bit in its memory.
                if effect.age == 1 {
                    let dealt = self.gore_critters(effect, effect.pos, radius, Fx::ONE);
                    let owed = effect.leeched(dealt);
                    self.players[effect.owner as usize].heal(owed);
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
                    // And at a small body anywhere along the line -- the same
                    // line `on_the_line` holds a fighter to. It cuts; it does
                    // not hold one (`docs/design/critters.md`).
                    let dealt = self.gore_critters_by(
                        effect,
                        EffectReach::Line(effect.pos, far, radius),
                        Fx::ONE,
                    );
                    let owed = effect.leeched(dealt);
                    self.players[effect.owner as usize].heal(owed);
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

            // A column of rising air. Everybody in it goes up once -- no
            // stun; the move's own disc gave the first ones theirs -- and on
            // its first frame every stone in it is lofted.
            EffectKind::Updraft => {
                let slab = effect.draft_volume();
                let (radius, height) = (t::body_radius(), t::body_height());
                for i in 0..MAX_PLAYERS {
                    let p = self.players[i];
                    if self.effects_reach(i, effect.owner)
                        && !effect.already_hit(0, i)
                        && slab.contains(effect.pos, p.pos, radius, height)
                    {
                        effect.take_hit(0, i);
                        self.players[i].vel.y = self.players[i].vel.y.max(t::updraft_lift());
                        self.players[i].grounded = false;
                    }
                }
                self.ride_the_thermal(effect);
                self.burn_in_the_draft(effect);
                if effect.age == 1 {
                    let inside = stones::within(&self.players, effect.pos, effect.reach);
                    for (index, hit) in inside.into_iter().enumerate() {
                        if hit {
                            stones::loft(&mut self.players, index, t::updraft_stone_lift());
                        }
                    }
                }
            }

            // A column of falling air, under her. Airborne bodies in it are
            // held at the drive -- the disc spiked them on the first frame,
            // this keeps them coming -- and stones in it come down: a lofted
            // one driven to the floor, a standing one pressed into it, leaving
            // broken ground where it stood. Once per stone, on the bits
            // `banked` lends for it.
            EffectKind::Downdraft => {
                self.burn_in_the_draft(effect);
                let slab = effect.draft_volume();
                let (radius, height) = (t::body_radius(), t::body_height());
                let drive = Fx::ZERO.sub(t::downdraft_drive());
                for i in 0..MAX_PLAYERS {
                    let p = self.players[i];
                    if self.effects_reach(i, effect.owner)
                        && !p.grounded
                        && slab.contains(effect.pos, p.pos, radius, height)
                    {
                        self.players[i].vel.y = self.players[i].vel.y.min(drive);
                    }
                }
                let inside = stones::within(&self.players, effect.pos, effect.reach);
                for (index, hit) in inside.into_iter().enumerate() {
                    let bit = 1i32 << index;
                    if !hit || effect.banked & bit != 0 {
                        continue;
                    }
                    if stones::resting(&self.players, index) {
                        effect.banked |= bit;
                        if let Some(middle) = stones::destroy(&mut self.players, index) {
                            let across = t::structure_radius().mul(Fx::from_int(2));
                            let owner = self.players[effect.owner as usize];
                            let from = V3::new(
                                middle.x.sub(owner.facing.x.mul(t::structure_radius())),
                                self.terrain().ground_under(middle),
                                middle.z.sub(owner.facing.z.mul(t::structure_radius())),
                            );
                            spawn_effect(
                                &mut self.effects,
                                Effect::cast(
                                    EffectKind::Rough,
                                    effect.owner,
                                    effect.class,
                                    SLOT_COMMITTED,
                                    from,
                                    owner.facing,
                                    across,
                                ),
                            );
                        }
                    } else {
                        stones::press(&mut self.players, index, t::downdraft_drive());
                    }
                }
            }

            // The air breaking outward from her feet: one shove, on the first
            // frame, and then only the picture. A brief stagger goes with it,
            // because a fighter free to act sets his own velocity from the
            // stick and would erase the shove before it moved him.
            EffectKind::AirRing => {
                if effect.age != 1 {
                    return;
                }
                let radius = t::air_ring_radius();
                for i in 0..MAX_PLAYERS {
                    let p = self.players[i];
                    let apart = V3::new(
                        p.pos.x.sub(effect.pos.x),
                        Fx::ZERO,
                        p.pos.z.sub(effect.pos.z),
                    );
                    if !self.effects_reach(i, effect.owner)
                        || apart.flat_len().raw() > radius.add(t::body_radius()).raw()
                    {
                        continue;
                    }
                    let away = if apart.flat_len().raw() > 0 {
                        apart.normalized()
                    } else {
                        self.players[effect.owner as usize].facing
                    };
                    let (guarding, parried) = guard_against(&p, effect.pos, true);
                    apply_hit(
                        &mut self.players[i],
                        Hit {
                            damage: 0,
                            hitstun: t::air_ring_stagger(),
                            blockstun: t::air_ring_stagger(),
                            knockback: t::air_ring_push(),
                            launch: Fx::ZERO,
                            grabs: 0,
                            by: effect.owner,
                            dir: away,
                            blocked: guarding,
                            parried,
                            interrupts: true,
                        },
                    );
                }
            }

            // A ring of fire racing outward: each body it passes takes one
            // hit and is shoved along with it.
            EffectKind::FireRing | EffectKind::ShadowRing => {
                let shadow = effect.kind == EffectKind::ShadowRing;
                let (width, stagger, push) = if shadow {
                    (
                        t::shadow_ring_width(),
                        t::shadow_ring_stagger(),
                        t::shadow_ring_push(),
                    )
                } else {
                    (
                        t::fire_ring_width(),
                        t::fire_ring_stagger(),
                        t::fire_ring_push(),
                    )
                };
                let r = effect.ring_radius();
                let band = width.add(t::body_radius());
                for i in 0..MAX_PLAYERS {
                    let p = self.players[i];
                    let apart = V3::new(
                        p.pos.x.sub(effect.pos.x),
                        Fx::ZERO,
                        p.pos.z.sub(effect.pos.z),
                    );
                    let dist = apart.flat_len();
                    if !self.effects_reach(i, effect.owner)
                        || effect.already_hit(0, i)
                        || dist.sub(r).abs().raw() > band.raw()
                        || !p.grounded && p.pos.y.raw() > effect.pos.y.add(t::body_height()).raw()
                    {
                        continue;
                    }
                    effect.take_hit(0, i);
                    let away = if dist.raw() > 0 {
                        apart.normalized()
                    } else {
                        self.players[effect.owner as usize].facing
                    };
                    let (guarding, parried) = guard_against(&p, effect.pos, false);
                    apply_hit(
                        &mut self.players[i],
                        Hit {
                            damage: effect.damage(),
                            hitstun: stagger,
                            blockstun: stagger,
                            knockback: push,
                            launch: Fx::ZERO,
                            grabs: 0,
                            by: effect.owner,
                            dir: away,
                            blocked: guarding,
                            parried,
                            interrupts: true,
                        },
                    );
                    if parried {
                        self.players[i].parried = PARRY_FLOURISH;
                    }
                }
                // The Plunge's ring reaches the creature and the small bodies
                // too, once each, inside its live radius: what it is for is
                // the feeling of the blow landing, and a hunt is most of the
                // game.
                if shadow {
                    let reach = r.add(width);
                    for slot in 0..MAX_MONSTERS {
                        self.gore_for_a_share(slot, effect, 0, effect.pos, reach, Fx::ONE);
                    }
                    self.gore_critters_by(effect, EffectReach::Column(effect.pos, reach), Fx::ONE);
                }
            }

            // The Reaver's shadow spinning (the Whirl): a ball round its
            // middle, live, taking a fresh pass at everybody on its beat. Its
            // first cut on a body marks it, once a spin.
            EffectKind::ShadowWhirl => {
                let every = t::whirl_every().max(1);
                let beat = effect.age > 1 && (effect.age - 1) % every == 0;
                if beat {
                    effect.forget_hits();
                }
                let radius = effect.field_radius();
                let mid = effect.pos.add(V3::new(
                    Fx::ZERO,
                    t::body_height().mul(Fx::ratio(1, 2)),
                    Fx::ZERO,
                ));
                for i in 0..MAX_PLAYERS {
                    if !self.effects_reach(i, effect.owner)
                        || effect.already_hit(0, i)
                        || !self.inside(i, mid, radius)
                    {
                        continue;
                    }
                    effect.take_hit(0, i);
                    let dealt = self.cut(i, effect, mid, effect.damage());
                    if dealt > 0 && marks_once(effect, i) {
                        shadow::mark(&mut self.players[i]);
                    }
                }
                // The creature and the small bodies are asked about an upright
                // column, a fighter tall, centred on the ball's middle: the
                // crosshair's point is in it, and so is a knee-high body under
                // a whirl on the floor.
                let low = V3::new(
                    mid.x,
                    mid.y.sub(t::body_height().mul(Fx::ratio(1, 2))),
                    mid.z,
                );
                self.shadow_blades_on_the_creatures(effect, 0, low, radius);
                // The small bodies on the beat (and the first frame) -- they
                // remember a cut themselves.
                if beat || effect.age == 1 {
                    self.gore_critters(effect, low, radius, Fx::ONE);
                }
            }

            // **The burst** as she collects her shadow: a thin disc round her
            // waist, each body once, and marked.
            EffectKind::ShadowBurst => {
                let radius = effect.field_radius();
                let band = t::disc_thickness();
                let mid = effect.pos.add(V3::new(
                    Fx::ZERO,
                    t::body_height().mul(Fx::ratio(1, 2)),
                    Fx::ZERO,
                ));
                for i in 0..MAX_PLAYERS {
                    let q = self.players[i];
                    let across =
                        V3::new(q.pos.x.sub(mid.x), Fx::ZERO, q.pos.z.sub(mid.z)).flat_len();
                    let level = mid.y.raw() >= q.pos.y.sub(band).raw()
                        && mid.y.raw() <= q.pos.y.add(q.hurt_height()).add(band).raw();
                    if !self.effects_reach(i, effect.owner)
                        || effect.already_hit(0, i)
                        || !level
                        || across.raw() > radius.add(t::body_radius()).raw()
                    {
                        continue;
                    }
                    effect.take_hit(0, i);
                    let dealt = self.cut(i, effect, mid, effect.damage());
                    if dealt > 0 && marks_once(effect, i) {
                        shadow::mark(&mut self.players[i]);
                    }
                }
                let low = V3::new(mid.x, mid.y.sub(band), mid.z);
                self.shadow_blades_on_the_creatures(effect, 0, low, radius);
                if effect.age == 1 {
                    self.gore_critters(effect, low, radius, Fx::ONE);
                }
            }

            // Thrown blades, each its own part: a body is cut once by each
            // blade that reaches it, and marked once by the throw.
            EffectKind::ShadowFan => {
                let radius = effect.field_radius();
                for blade in 0..moves::reaver::blades(effect.slot) {
                    let at = effect.fan_blade_at(blade);
                    for i in 0..MAX_PLAYERS {
                        if !self.effects_reach(i, effect.owner)
                            || effect.already_hit(blade, i)
                            || !self.inside(i, at, radius)
                        {
                            continue;
                        }
                        effect.take_hit(blade, i);
                        let dealt = self.cut(i, effect, at, effect.damage());
                        if dealt > 0 && marks_once(effect, i) {
                            shadow::mark(&mut self.players[i]);
                        }
                    }
                    self.shadow_blades_on_the_creatures(effect, blade, at, radius);
                    self.gore_critters(effect, at, radius, Fx::ONE);
                }
            }

            // The shake: anybody **moving** through it, on the floor, is
            // staggered once. Standing still is the answer, and it is the
            // whole read. The eruption is `pay_out`'s, when the shake ends.
            EffectKind::Quake => {
                let radius = effect.field_radius().add(t::body_radius());
                for i in 0..MAX_PLAYERS {
                    let p = self.players[i];
                    let apart = V3::new(
                        p.pos.x.sub(effect.pos.x),
                        Fx::ZERO,
                        p.pos.z.sub(effect.pos.z),
                    );
                    if !self.effects_reach(i, effect.owner)
                        || effect.already_hit(0, i)
                        || !p.grounded
                        || apart.flat_len().raw() > radius.raw()
                        || p.vel.flat_len().raw() <= t::quake_still_speed().raw()
                    {
                        continue;
                    }
                    effect.take_hit(0, i);
                    apply_hit(
                        &mut self.players[i],
                        Hit {
                            damage: 0,
                            hitstun: t::quake_stagger(),
                            blockstun: t::quake_stagger(),
                            knockback: Fx::ZERO,
                            launch: Fx::ZERO,
                            grabs: 0,
                            by: effect.owner,
                            dir: V3::ZERO,
                            blocked: false,
                            parried: false,
                            interrupts: true,
                        },
                    );
                }
            }

            // Broken ground: felt through the feet, every frame, by anybody
            // crossing it. Never the one who broke it -- her own scar slowing
            // her would make the crack a thing thrown at herself.
            EffectKind::Rough => {
                let radius = t::body_radius();
                for i in 0..MAX_PLAYERS {
                    let p = self.players[i];
                    if self.effects_reach(i, effect.owner)
                        && effect.roughens(p.pos, p.grounded, radius)
                    {
                        self.players[i].slow(t::slow_frames(), t::rough_slow());
                    }
                }
            }

            // A cloud of embers burns like a pillar does, on the same tick,
            // inside the ball it occupies -- gently, because what it is for is
            // lighting the shots that fly through it, and the burn is only
            // what makes standing in one a mistake.
            EffectKind::Embers => {
                if effect.ticks_now() {
                    let slab = effect.ember_volume();
                    let radius = t::body_radius();
                    let height = t::body_height();
                    for i in 0..MAX_PLAYERS {
                        let p = self.players[i];
                        if self.effects_reach(i, effect.owner)
                            && slab.contains(effect.pos, p.pos, radius, height)
                        {
                            self.drain(i, effect);
                        }
                    }
                    self.feed_the_caster(effect, 0, effect.pos, slab.radius);
                }
            }

            // The Air ball rolls, carries, and shrinks -- all of it in one
            // place, because each of the three reads the other two. It hurts
            // nobody. See `roll_the_ball`.
            EffectKind::AirBall => self.roll_the_ball(effect),

            // The Fire carpet burns whoever touches the strip, on the tick.
            // What it is *for* is the shot flown down it and the Thermal
            // thrown along it; the burn is what makes it a thing in the way.
            EffectKind::FireCarpet => {
                self.sink_the_carpet(effect);
                if effect.ticks_now() {
                    let (a, b) = effect.carpet_line();
                    let radius = effect.field_radius();
                    for i in 0..MAX_PLAYERS {
                        if self.effects_reach(i, effect.owner) && self.on_the_line(i, a, b, radius)
                        {
                            self.drain(i, effect);
                        }
                    }
                    // The small bodies along the same line a fighter is held
                    // to, and each creature where the strip passes nearest
                    // its middle -- one burn a tick, wherever it is touched.
                    self.gore_critters_by(effect, EffectReach::Line(a, b, radius), Fx::ONE);
                    for slot in 0..MAX_MONSTERS {
                        let Some(beast) = self.monsters[slot] else {
                            continue;
                        };
                        let (along, _, _) =
                            crate::math::segment_closest(a, b, beast.pos, beast.pos);
                        let nearest = crate::math::lerp3(a, b, along);
                        let dealt =
                            self.gore_for_a_share(slot, effect, 0, nearest, radius, Fx::ONE);
                        let owed = effect.leeched(dealt);
                        self.players[effect.owner as usize].heal(owed);
                    }
                }
            }

            // The Fire fountain: its first frames are the burst at her feet,
            // the move's own hit once per body; after that a wash of fire
            // standing where she took off, burning on the tick.
            EffectKind::Fountain => {
                let wash = effect.fountain_volume();
                let (radius, height) = (t::body_radius(), t::body_height());
                if effect.age <= t::fountain_burst() {
                    let burst = effect.source().radius;
                    for i in 0..MAX_PLAYERS {
                        if self.effects_reach(i, effect.owner)
                            && !effect.already_hit(0, i)
                            && self.inside(i, effect.pos, burst)
                        {
                            effect.take_hit(0, i);
                            self.cut(i, effect, effect.pos, effect.source().damage);
                        }
                    }
                    self.gore_the_creature(effect, 0, effect.pos, burst);
                } else if effect.ticks_now() {
                    for i in 0..MAX_PLAYERS {
                        let p = self.players[i];
                        if self.effects_reach(i, effect.owner)
                            && wash.contains(effect.pos, p.pos, radius, height)
                        {
                            self.drain(i, effect);
                        }
                    }
                }
            }

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
                // **The creature bleeds** (2026-10-09): what the bolt cuts on
                // it opens a bleed, ticking and spilling under it as a
                // fighter's does. See `World::bleed_the_creatures`.
                let cut = self.gore_each(effect, 0, at, radius, Fx::ONE);
                for (slot, dealt) in cut.iter().enumerate() {
                    if *dealt > 0 {
                        spent = true;
                        if t::bleed_on_creatures() {
                            if let Some(beast) = self.monsters[slot].as_mut() {
                                beast.bleeding = beast.bleeding.max(t::bleed_lasts());
                                beast.bled_by = effect.owner;
                            }
                        }
                    }
                }
                if self.gore_critters(effect, at, radius, Fx::ONE) > 0 {
                    spent = true;
                }
                if spent {
                    effect.age = effect.life;
                }
            }

            // **Binary's twin stars** (2026-10-12): round her, live. Each
            // strikes a body it passes through once a window -- the dark star
            // (part 0) curses, the light star (part 1) purges at the power the
            // stars were raised with, her lower bar.
            EffectKind::Stars => {
                let owner = effect.owner as usize;
                let o = self.players[owner];
                if o.health <= 0 {
                    effect.age = effect.life;
                    return;
                }
                effect.home = V3::new(
                    o.pos.x,
                    o.pos.y.add(t::body_height().mul(Fx::ratio(1, 2))),
                    o.pos.z,
                );
                if effect.age % t::binary_every().max(1) == 0 {
                    effect.forget_hits();
                }
                let radius = effect.field_radius();
                for star in 0..2 {
                    let at = effect.star_at(star);
                    for i in 0..MAX_PLAYERS {
                        if !self.effects_reach(i, effect.owner)
                            || effect.already_hit(star, i)
                            || !self.inside(i, at, radius)
                        {
                            continue;
                        }
                        effect.take_hit(star, i);
                        let dealt = self.cut(i, effect, at, effect.damage());
                        if dealt > 0 {
                            dual::landed_a_hit(&mut self.players[owner]);
                            if star == 0 {
                                let v = &mut self.players[i];
                                dual::lay_curse(&mut v.curse, &mut v.curse_clock, owner, 1);
                            } else {
                                purge_the_fighter(&mut self.players, owner, i, effect.power);
                            }
                        }
                    }
                    let cut = self.gore_each(effect, star, at, radius, Fx::ONE);
                    for (slot, dealt) in cut.iter().enumerate() {
                        if *dealt > 0 {
                            if let Some(mut beast) = self.monsters[slot] {
                                dual::landed_a_hit(&mut self.players[owner]);
                                if star == 0 {
                                    dual::lay_curse(
                                        &mut beast.curse,
                                        &mut beast.curse_clock,
                                        owner,
                                        1,
                                    );
                                } else {
                                    purge_the_creature(
                                        &mut beast,
                                        &mut self.players[owner],
                                        effect.power,
                                    );
                                }
                                self.monsters[slot] = Some(beast);
                            }
                        }
                    }
                    // A critter is knee-high: the star passes over it at her
                    // waist, so for the small bodies its strike is a column
                    // up from the floor through it (bestiary A1).
                    let floor = effect.home.y.sub(t::body_height().mul(Fx::ratio(1, 2)));
                    self.gore_critters(effect, V3::new(at.x, floor, at.z), radius, Fx::ONE);
                }
            }

            // **The Abyss** (2026-10-12): a black sun. Whoever is under it is
            // pinned where they stand, ticks, drains back to her, and gains a
            // curse stack every `abyss, curses every`; on its last frame it
            // collapses in one more blow. It drags nobody anywhere.
            EffectKind::Abyss => {
                let radius = effect.field_radius();
                let owner = effect.owner as usize;
                let cursing = effect.age % t::abyss_curse_every().max(1) == 0;
                // The last frame the effect is applied on: it is gone before
                // `apply_effect` is asked on the next.
                let collapsing = effect.age.saturating_add(1) >= effect.life;
                for i in 0..MAX_PLAYERS {
                    if !self.effects_reach(i, effect.owner) || !self.inside(i, effect.pos, radius) {
                        continue;
                    }
                    if !self.players[i].untouchable() {
                        self.players[i].slow(2, t::root_slow());
                    }
                    if effect.ticks_now() {
                        let dealt = self.cut(i, effect, effect.pos, effect.damage());
                        if dealt > 0 {
                            self.players[owner].heal(effect.leeched(dealt));
                            dual::landed_a_hit(&mut self.players[owner]);
                        }
                    }
                    if cursing && !self.players[i].untouchable() {
                        let v = &mut self.players[i];
                        dual::lay_curse(&mut v.curse, &mut v.curse_clock, owner, 1);
                    }
                    if collapsing {
                        let dealt = self.cut(i, effect, effect.pos, t::abyss_collapse());
                        if dealt > 0 {
                            dual::landed_a_hit(&mut self.players[owner]);
                        }
                    }
                }
                for slot in 0..crate::monster::MAX_MONSTERS {
                    let Some(mut beast) = self.monsters[slot] else {
                        continue;
                    };
                    if !beast.alive()
                        || beast
                            .part_struck(effect.pos, radius, t::body_height())
                            .is_none()
                    {
                        continue;
                    }
                    root_the_creature(&mut beast, 2);
                    if cursing {
                        dual::lay_curse(&mut beast.curse, &mut beast.curse_clock, owner, 1);
                    }
                    self.monsters[slot] = Some(beast);
                }
                if effect.ticks_now() {
                    let cut = self.gore_each(effect, 0, effect.pos, radius, Fx::ONE);
                    for dealt in cut.iter() {
                        if *dealt > 0 {
                            self.players[owner].heal(effect.leeched(*dealt));
                            dual::landed_a_hit(&mut self.players[owner]);
                        }
                    }
                    // A tick is a fresh pass at the animal.
                    effect.forget_hits();
                    self.gore_critters(effect, effect.pos, radius, Fx::ONE);
                }
                if collapsing {
                    effect.forget_hits();
                    let share = Fx::ratio(t::abyss_collapse(), effect.damage().max(1));
                    self.gore_each(effect, 0, effect.pos, radius, share);
                }
            }

            // **A burst** (2026-10-12): one frame of hitting in a disc, at
            // whoever is inside it, once; the rest of its life is the picture.
            // Which burst it is is its slot. The Shadowstep's arrival lays its
            // curse; Phase's flash is only light; Judgement's outer columns
            // -- `banked` of them round `pos`, `judgement, columns stand
            // apart` out -- purge like the middle one; Apotheosis purges every
            // curse in it at the top of her power.
            EffectKind::Burst => {
                if effect.age != 1 {
                    return;
                }
                let owner = effect.owner as usize;
                let radius = effect.field_radius();
                let apotheosis = effect.slot == crate::effects::APOTHEOSIS;
                let damage = if apotheosis {
                    t::apotheosis_damage()
                } else {
                    effect.damage()
                };
                let centres: [Option<V3>; crate::effects::BURST_CENTRES] =
                    std::array::from_fn(|k| effect.burst_at(k));
                for at in centres.into_iter().flatten() {
                    for i in 0..MAX_PLAYERS {
                        // Judgement's middle column is the move's own blow: a
                        // fighter it already reaches is not struck again by
                        // the ring round it.
                        let in_the_middle = effect.slot == moves::dual::JUDGEMENT
                            && self.inside(i, effect.pos, radius);
                        if !self.effects_reach(i, effect.owner)
                            || effect.already_hit(0, i)
                            || in_the_middle
                            || !self.inside(i, at, radius)
                        {
                            continue;
                        }
                        effect.take_hit(0, i);
                        let dealt = self.cut(i, effect, at, damage);
                        if dealt > 0 {
                            dual::landed_a_hit(&mut self.players[owner]);
                        }
                        if apotheosis {
                            purge_the_fighter(&mut self.players, owner, i, t::depth_ceiling());
                        } else if dealt > 0 {
                            spell_lands_on_fighter(&mut self.players, owner, i, effect.slot);
                        }
                    }
                    let share = Fx::ratio(damage, effect.damage().max(1));
                    let cut = self.gore_each(effect, 0, at, radius, share);
                    for (slot, dealt) in cut.into_iter().enumerate() {
                        let Some(mut beast) = self.monsters[slot] else {
                            continue;
                        };
                        let inside = beast.alive()
                            && beast.part_struck(at, radius, t::body_height()).is_some();
                        if apotheosis && inside {
                            purge_the_creature(
                                &mut beast,
                                &mut self.players[owner],
                                t::depth_ceiling(),
                            );
                        } else if dealt > 0 {
                            dual::landed_a_hit(&mut self.players[owner]);
                            spell_lands_on_creature(
                                &mut beast,
                                &mut self.players,
                                owner,
                                effect.slot,
                            );
                        }
                        self.monsters[slot] = Some(beast);
                    }
                    self.gore_critters(effect, at, radius, share);
                }
            }

            // **A sigil** (2026-10-12): consecrated ground. Anybody else
            // standing in it burns on the tick; her own ward is the world's
            // business once a frame (`step_spellbook`), not the sigil's.
            EffectKind::Sigil => {
                if !effect.ticks_now() {
                    return;
                }
                // A burn, not a blow: Judgement's field's `drain`, which hurts
                // and does not stun.
                let radius = effect.field_radius();
                for i in 0..MAX_PLAYERS {
                    if self.effects_reach(i, effect.owner) && self.inside(i, effect.pos, radius) {
                        self.drain(i, effect);
                    }
                }
                effect.forget_hits();
                self.feed_the_caster(effect, 0, effect.pos, radius);
            }

            // The Blood nova: on its first frame, everyone inside is hurt and
            // thrown outward from where she stood; the creature is hurt.
            EffectKind::Nova => {
                if effect.age != 1 {
                    return;
                }
                let radius = effect.field_radius();
                let centre = V3::new(
                    effect.pos.x,
                    effect.pos.y.add(t::body_height().mul(Fx::ratio(1, 2))),
                    effect.pos.z,
                );
                // What it is worth: the Tides' tier, a burst's own number
                // when it carries one (an exploding stain), or the row's.
                let blow = nova_blow(effect);
                for i in 0..MAX_PLAYERS {
                    if self.effects_reach(i, effect.owner) && self.inside(i, centre, radius) {
                        self.cut(i, effect, effect.pos, blow);
                    }
                }
                let share = Fx::ratio(blow, effect.damage().max(1));
                self.gore_each(effect, 0, effect.pos, radius, share);
                self.gore_critters(effect, effect.pos, radius, share);
            }

            // The Nail: the Haemorrhage's flight, spent on the first body it
            // reaches. In the air, that body is pinned; on the floor, hit.
            EffectKind::Nail => {
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
                    let aloft = !self.players[i].grounded;
                    let dealt = self.cut(i, effect, at, effect.source().damage);
                    if dealt > 0 && aloft && t::nail_pin() > 0 {
                        let q = &mut self.players[i];
                        q.seized(NAILED, t::nail_pin(), 0);
                        q.vel = V3::ZERO;
                        q.grounded = false;
                    }
                    spent = true;
                }
                if self.gore_the_creature(effect, 0, at, radius) > 0 {
                    spent = true;
                }
                // **The Stake goes through** whoever it pins, on to the floor
                // at the end of its flight, where it lands (`pay_out`).
                if spent && effect.slot != moves::blood::STAKE {
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
                            // Somebody is in the arms, so they are not free to
                            // pull on anything else.
                            self.players[effect.owner as usize].haul = 0;
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
        victim as u8 != owner
            && self.players[victim].health > 0
            && self.pvp()
            && (owner as usize >= MAX_PLAYERS || self.foes(owner as usize, victim))
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
        if p.untouchable() {
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
        if p.untouchable() || (p.crouching && !m.hits_crouching) {
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
        // A Champion's finisher's effect staggers by the momentum it spent,
        // which it carries in `banked` (exploration 0015).
        if dealt > 0 && effect.class == Class::Champion && effect.banked > 0 {
            let spent = effect.banked.clamp(0, 100) as u8;
            stagger_by_the_spend(&mut self.players[victim], m.hitstun, spent);
        }
        // The Blood mage's double duty, for a hit her effects deliver: drink
        // from a pool the victim stands in, then spill what was dealt. See the
        // melee loop in `advance` for the order. **Not for the blade**: it
        // drinks on its way home, from each pool it crosses, once -- a cut
        // that also drank would take three shares of one pool in one throw.
        let mut dealt = dealt;
        if dealt > 0 {
            if !effect.kind.comes_home() {
                let bonus = self.stain_bonus(
                    effect.owner as usize,
                    &m,
                    from,
                    from,
                    Some(victim),
                    effect.age,
                );
                dealt += self.players[victim].wound(bonus);
            }
            if !effect.spills_nothing() {
                self.spill_under(effect.owner, effect.class, effect.slot, &p, dealt);
            } else {
                // An eruption spills nothing, but it was her blow.
                self.credit(effect.owner, dealt);
            }
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
        let creatures: i32 = self
            .gore_each(effect, part, at, radius, Fx::ONE)
            .iter()
            .sum();
        creatures + self.gore_critters(effect, at, radius, Fx::ONE)
    }

    /// An effect landing on the small bodies inside it: an upright column at
    /// `at`, `radius` wide and a fighter tall, against each critter's box --
    /// the same column the creature's parts are tested against.
    ///
    /// **A blade cuts a critter once per throw**: the critter remembers it
    /// (`Critter::seared`), because an effect's memory of what it has struck
    /// has no bits to spare for ten more victims. So a blade out and home is
    /// one cut on a critter, where it is two on a fighter. A field ticks, as
    /// it does on everything. See `docs/design/critters.md`.
    fn gore_critters(&mut self, effect: &mut Effect, at: V3, radius: Fx, share: Fx) -> i32 {
        self.gore_critters_by(effect, EffectReach::Column(at, radius), share)
    }

    /// [`Self::gore_critters`] for any of the shapes an effect reaches with.
    fn gore_critters_by(&mut self, effect: &mut Effect, reach: EffectReach, share: Fx) -> i32 {
        if !self.critters.any() || self.pack.is_none() {
            return 0;
        }
        let once = effect.kind.once_a_pass();
        let sp = self.critters.sp();
        let raw = Fx::from_int(effect.damage())
            .mul(share)
            .mul(preying(effect.class, false))
            .to_int();
        let mut total = 0;
        for c in 0..critter::MAX_CRITTERS {
            let body = self.critters[c];
            if !body.alive() || (once && body.seared_by(effect.owner, effect.slot)) {
                continue;
            }
            let shape = body.body(sp);
            let (inside, from) = match reach {
                EffectReach::Column(at, radius) => {
                    (shape.touched_by_column(at, radius, t::body_height()), at)
                }
                EffectReach::Line(a, b, radius) => {
                    (shape.gap_to_segment(a, b).raw() <= radius.raw(), a)
                }
                EffectReach::Slab(a, b, radius, half) => {
                    (shape.touched_by_slab(a, b, radius, half), effect.pos)
                }
            };
            if !inside {
                continue;
            }
            if once {
                self.critters[c].sear(effect.owner, effect.slot, true);
            }
            let away = body.pos.sub(from);
            let away = V3::new(away.x, Fx::ZERO, away.z).normalized();
            let dealt = pack::hurt(
                &mut self.pack,
                &mut self.critters,
                c,
                raw,
                away,
                Fx::ZERO,
                Fx::ZERO,
            );
            if dealt > 0 && effect.spills_nothing() {
                self.credit(effect.owner, dealt);
            }
            if dealt > 0 && !effect.spills_nothing() {
                let m = effect.source();
                self.drink_over(effect.owner as usize, &m, from, from, None, effect.age);
                // A small body bleeds what the blow was worth, not the little
                // it had left to lose.
                self.spill(
                    effect.owner,
                    effect.class,
                    effect.slot,
                    floor_under(self.arena(), body.pos),
                    dealt.max(raw),
                );
            }
            total += dealt;
        }
        total
    }

    /// Every creature, in slot order, and what each took.
    fn gore_each(
        &mut self,
        effect: &mut Effect,
        part: usize,
        at: V3,
        radius: Fx,
        share: Fx,
    ) -> [i32; MAX_MONSTERS] {
        std::array::from_fn(|slot| self.gore_for_a_share(slot, effect, part, at, radius, share))
    }

    /// [`Self::gore_the_creature`] for `share` of the effect's damage: the
    /// lotus coming home, which deals the creature the same fraction of its
    /// way out that it deals a fighter.
    fn gore_for_a_share(
        &mut self,
        slot: usize,
        effect: &mut Effect,
        part: usize,
        at: V3,
        radius: Fx,
        share: Fx,
    ) -> i32 {
        let victim = quarry_victim(slot);
        if effect.already_hit(part, victim) {
            return 0;
        }
        let Some(mut beast) = self.monsters[slot] else {
            return 0;
        };
        if !beast.alive() {
            return 0;
        }
        let Some(struck) = beast.part_struck(at, radius, t::body_height()) else {
            return 0;
        };
        let raw = Fx::from_int(effect.damage())
            .mul(share)
            .mul(preying(effect.class, beast.disabled()))
            .to_int();
        // A stain under the struck part is drunk, and the blow is harder for
        // it -- asked before the blow, so the creature takes the whole.
        let raw = if raw > 0 {
            let m = effect.source();
            raw + self.stain_bonus(effect.owner as usize, &m, at, at, None, effect.age)
        } else {
            raw
        };
        // Asked of its guard from where the effect is: a burst behind a
        // guard, or a pillar under it, comes from outside it.
        let (dealt, guarded) = beast.take_blow(
            struck,
            raw,
            &monster::Blow {
                from: at,
                unblockable: effect.source().unblockable,
                who: effect.owner,
                class: effect.class,
                kind: effect.slot,
            },
        );
        // The same control a fighter would have taken, offered rather than
        // applied: the creature decides how much of it it is currently in a
        // state to feel. See `monster::Monster::take_control`.
        if guarded == monster::Guarded::Lands {
            beast.take_control(effect.control());
            // The Champion's full charges weigh on it like his blows do, and
            // the Meteor's crater knocks it off its feet (exploration 0015).
            if effect.class == Class::Champion && (effect.owner as usize) < MAX_PLAYERS {
                impact(
                    &mut beast,
                    &self.players[effect.owner as usize],
                    effect.slot,
                    dealt,
                );
                if effect.slot == moves::champion::METEOR && dealt > 0 {
                    beast.stumble_for(t::meteor_stumble());
                }
            }
        }
        self.monsters[slot] = Some(beast);
        // A field has one part and hits over and over on its tick; a blade or an
        // arm has a pass to spend and spends it here.
        if effect.kind.once_a_pass() {
            effect.take_hit(part, victim);
        }
        // The creature bleeds too: under the struck part, projected to the
        // floor, which is what makes the pool under a toppled Ridgeback a
        // door onto its back.
        // **Every hit on a creature leaves a stain** (2026-10-12), sized by
        // what the blow was worth rather than by what its hide let through:
        // a hit on a plate still opens it.
        if dealt > 0 && effect.spills_nothing() {
            self.credit(effect.owner, dealt);
        }
        if (dealt > 0 || (raw > 0 && !guarded.turned())) && !effect.spills_nothing() {
            self.spill(
                effect.owner,
                effect.class,
                effect.slot,
                floor_under(self.arena(), at),
                dealt.max(raw),
            );
        }
        dealt
    }
}

/// The shape an effect reaches a small body with: an upright column on the
/// floor (a field, a burst, a blade's bubble), a line (a tether looking for
/// something), or a level blade's slab (the Guillotine).
#[derive(Clone, Copy)]
enum EffectReach {
    Column(V3, Fx),
    Line(V3, V3, Fx),
    Slab(V3, V3, Fx, Fx),
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
        if far.raw() > t::teleport_reach().raw() {
            continue;
        }
        if best.is_none_or(|(_, seen)| far.raw() < seen.raw()) {
            best = Some((slot, far));
        }
    }
    best.map(|(slot, _)| slot)
}

/// Which of the Elementalist's own stones a dodge along `dir` goes into, with
/// the crosshair on it, if any -- the nearest, when it is on more than one.
///
/// The same question the Blood mage's blink asks of her pools, asked of the
/// class's object: the crosshair on the stone's standing column, with some
/// slack, and the stone near enough that the dodge carries her into it. The
/// dodge has to be *toward* the stone as well: the camera sits behind her
/// shoulder, and turned away from a stone at her back it can be inside the
/// stone's column with the crosshair reading as on it -- a dodge away from a
/// stone is a dodge, not a demolition. Hers only -- the Bulwark's planted
/// wall stands in the same field and is nobody's to break through.
fn stone_under_the_crosshair(
    p: &Player,
    who: usize,
    input: Input,
    dir: V3,
    scene: &Scene,
) -> Option<usize> {
    let Mechanic::Structures(slots) = p.mechanic else {
        return None;
    };
    let mut best: Option<(usize, Fx)> = None;
    for (slot, stone) in slots.iter().enumerate() {
        let Some(stone) = stone else { continue };
        let apart = V3::new(stone.at.x.sub(p.pos.x), Fx::ZERO, stone.at.z.sub(p.pos.z));
        let far = apart.flat_len();
        if apart.dot(dir).raw() <= 0
            || far.raw() > t::break_reach().add(t::structure_radius()).raw()
            || !aim::pointing_at(
                who,
                input,
                stone.at,
                t::break_lock().add(t::structure_radius()),
                scene,
            )
        {
            continue;
        }
        if best.is_none_or(|(_, seen)| far.raw() < seen.raw()) {
            best = Some((slot, far));
        }
    }
    best.map(|(slot, _)| slot)
}

/// The patch of floor under a point in the air: the arena's ground, and the
/// creature's contact points are always over it. Stones are not consulted --
/// a pool spilled onto a raised structure is an open question in the design,
/// and until it is answered blood falls to the floor.
fn floor_under(arena: &arena::Arena, at: V3) -> V3 {
    V3::new(at.x, arena.relief_at(at.x, at.z), at.z)
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
        // **In the air, it falls**: on to the floor under them, since
        // 2026-10-09. Marionette and the Nail put people in the air, and a hit
        // up there that spilled nowhere would starve the class of the very
        // blood its combos run on. See `docs/design/exploration/0009`.
        let at = if victim.grounded {
            victim.pos
        } else {
            V3::new(
                victim.pos.x,
                self.terrain().ground_under(victim.pos),
                victim.pos.z,
            )
        };
        self.spill(owner, class, slot, at, volume);
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
        // Every hit she lands spills, and what it spills is what it was
        // worth: the one place her damage is counted toward a heart.
        self.credit(owner, volume);
        self.pool_blood(owner, class, slot, at, volume);
    }

    /// **Damage she dealt, counted toward a heart** (2026-10-18), up to
    /// `tuning::heart_meter_full`.
    fn credit(&mut self, owner: u8, dealt: i32) {
        let Some(p) = self.players.get_mut(owner as usize) else {
            return;
        };
        if p.class != Class::BloodMage || dealt <= 0 {
            return;
        }
        let full = t::heart_meter_full().clamp(0, u16::MAX as i32);
        p.heart_meter = (p.heart_meter as i32 + dealt).min(full) as u16;
    }

    /// Put `volume` of blood on the floor at `at` as [`Self::spill`] does,
    /// without counting it as damage dealt: what a cast cost her, and what a
    /// thrall had left when it was killed.
    fn pool_blood(&mut self, owner: u8, class: Class, slot: u8, at: V3, volume: i32) {
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
        // **Past the cap, her oldest stain makes way** (2026-10-19), and
        // bleeds into her ground where it stood as a stain that fades does.
        // It used to be the new blood that gave way, poured into her newest
        // stain wherever that was, so once the cap filled no new stain ever
        // appeared where blood was spilled. A lit stain is going up and is
        // left to.
        let mut count = 0;
        let mut oldest: Option<(usize, u16)> = None;
        for (i, e) in self.effects.iter().enumerate() {
            let Some(e) = e else { continue };
            if !e.is_a_pool() || e.owner != owner {
                continue;
            }
            count += 1;
            if e.life == UNLIT && oldest.is_none_or(|(_, age)| e.age > age) {
                oldest = Some((i, e.age));
            }
        }
        if count >= t::pool_cap() {
            let Some((i, _)) = oldest else {
                return;
            };
            if let Some(gone) = self.effects[i].take() {
                self.lay_ground(
                    gone.owner,
                    gone.pos,
                    gone.stain_ground(),
                    t::ground_stain_lasts(),
                );
            }
        }
        spawn_effect(&mut self.effects, born);
    }

    /// The pool of `owner`'s that a point on the floor is inside, if any.
    fn pool_covering(&self, owner: u8, at: V3) -> Option<usize> {
        self.effects
            .iter()
            .position(|e| e.is_some_and(|e| e.is_a_pool() && e.owner == owner && e.covers(at)))
    }

    /// The nearest unlit stain of `owner`'s that a spike's disc of `reach` at
    /// `at` touches -- not only one under its middle (2026-10-19): the spike
    /// is the finisher of what a bleed laid, and one that lands a stride
    /// short of a stain sets it off rather than drinking it in passing.
    fn pool_touching(&self, owner: u8, at: V3, reach: Fx) -> Option<usize> {
        let mut best: Option<(usize, i32)> = None;
        for (slot, e) in self.effects.iter().enumerate() {
            let Some(e) = e else { continue };
            if !e.is_a_pool() || e.owner != owner || e.life != UNLIT {
                continue;
            }
            let far = V3::new(at.x.sub(e.pos.x), Fx::ZERO, at.z.sub(e.pos.z)).flat_len();
            let level = at.y.sub(e.pos.y).abs().raw() <= e.pool_height().raw();
            if level
                && far.raw() <= e.pool_radius().add(reach).raw()
                && best.is_none_or(|(_, b)| far.raw() < b)
            {
                best = Some((slot, far.raw()));
            }
        }
        best.map(|(slot, _)| slot)
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
    ///
    /// Gives back the **essence** of the stain it drank -- what was in it, not
    /// what came back -- or nothing; what a hit through it adds is read off
    /// that (`stain_bonus`).
    ///
    /// `older_than` is the age of the thing delivering the hit, and only pools
    /// older than it count. Nought for a swing, whose one hit has nothing of
    /// its own to find; for an effect it is the effect's own age, so four arms
    /// closing on one spot cannot each drink what the arm before it spilled --
    /// which was, for a frame, a Grasp that refunded itself.
    fn drink_over(
        &mut self,
        owner: usize,
        m: &moves::Move,
        from: V3,
        to: V3,
        victim: Option<usize>,
        older_than: u16,
    ) -> i32 {
        if m.drink == 0 {
            return 0;
        }
        let feet = victim.map(|v| self.players[v].pos);
        let mut best: Option<(usize, i32)> = None;
        for (slot, e) in self.effects.iter().enumerate() {
            let Some(e) = e else { continue };
            if !e.is_a_pool()
                || e.owner != owner as u8
                || e.age <= older_than.max(t::stain_settles())
            {
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
            Some((slot, essence)) => {
                self.drink_from(slot, owner, m);
                essence
            }
            None => 0,
        }
    }

    /// [`Self::drink_over`], and what the stain it drank adds to the blow
    /// that went through it (`tuning::stain_hit_bonus`): every attack of hers
    /// that lands over a stain heals her **and** hits harder for it.
    fn stain_bonus(
        &mut self,
        owner: usize,
        m: &moves::Move,
        from: V3,
        to: V3,
        victim: Option<usize>,
        older_than: u16,
    ) -> i32 {
        let essence = self.drink_over(owner, m, from, to, victim, older_than);
        t::stain_hit_bonus(essence)
    }

    /// **Walking into a stain drinks it** (2026-10-12): every stain of hers --
    /// not her own blood -- that her feet are against is drunk, at
    /// `tuning::stain_walk_drinks` of its essence. The Demon Avenger's orbs:
    /// the heal is a place, and going there is enough.
    ///
    /// **Only what she can use.** Walking is not a choice the way a blow is,
    /// so it takes no more than her grey can hold and the stain keeps the
    /// rest: at full health she can walk through her stains and leave them
    /// standing for the spike.
    fn drink_underfoot(&mut self) {
        for i in 0..MAX_PLAYERS {
            let p = self.players[i];
            if p.health <= 0 || !p.class.wounds_go_grey() {
                continue;
            }
            for slot in 0..MAX_EFFECTS {
                let Some(e) = self.effects[slot] else {
                    continue;
                };
                // A lit stain is going up, not to be drunk.
                if !e.is_a_pool()
                    || e.owner != i as u8
                    || e.age <= t::stain_settles()
                    || e.life != UNLIT
                    || !e.covers(p.pos)
                {
                    continue;
                }
                // **Only what her grey can hold** (2026-10-19): with no Well to
                // pour the rest into, a stain she walks through at full health
                // stays on the floor -- her ground, and a spike's.
                let share = t::stain_walk_drinks().max(1);
                let want = e.banked * share / 100;
                let took = self.players[i].absorb(want);
                if took >= want {
                    self.effects[slot] = None;
                } else if let Some(pool) = self.effects[slot].as_mut() {
                    pool.banked -= took * 100 / share;
                }
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
        for slot in 0..MAX_EFFECTS {
            let over = self.effects[slot].is_some_and(|e| {
                e.is_a_pool()
                    && e.owner == who as u8
                    && e.age > t::stain_settles()
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
        let got = self.players[owner].absorb(take);
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
            // Held by nobody: the creature's spikes, which pin a fighter
            // where they stand rather than haul them anywhere. The pin is
            // the whole of it -- no momentum of their own, and whatever
            // the hit carried stops at the spikes.
            victim.vel.x = Fx::ZERO;
            victim.vel.z = Fx::ZERO;
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
            // Arrived, this frame, if there was any ground left to cover.
            victim.hauled_in = far.raw() > 0;
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
    spawn_effect_at(effects, effect);
}

/// [`spawn_effect`], and which slot it went into, if any.
fn spawn_effect_at(effects: &mut [Option<Effect>; MAX_EFFECTS], effect: Effect) -> Option<usize> {
    let owner = effect.owner;
    if let Some(free) = effects.iter().position(|s| s.is_none()) {
        effects[free] = Some(effect);
        return Some(free);
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
    oldest
}

/// Put an effect into the world in place of any of the same kind its owner
/// already has: one Air ball, one carpet, one wash of fire at a time.
/// **The creature notices her thralls** (2026-10-13) **and her hearts**
/// (2026-10-18): what it is shown of the Blood mage is the nearest of them
/// when one is nearer it than she is -- so the creature turns on the body in
/// its way, and its blows land on that (`blow_on_a_thrall`). Not while she
/// is aboard it.
fn decoy_for(
    effects: &[Option<Effect>; MAX_EFFECTS],
    i: usize,
    beast: &Monster,
    real: Quarry,
) -> Quarry {
    if !real.alive || real.aboard {
        return real;
    }
    let near = |at: V3| {
        let c = beast.nearest_to(at);
        V3::new(at.x.sub(c.x), Fx::ZERO, at.z.sub(c.z)).flat_len()
    };
    let mine = near(real.pos);
    let mut best: Option<(V3, Fx)> = None;
    for e in effects.iter().flatten() {
        if !(e.is_a_thrall() || e.is_a_heart()) || e.owner as usize != i {
            continue;
        }
        let far = near(e.pos);
        if far.raw() >= mine.raw() {
            continue;
        }
        if best.is_none_or(|(_, b)| far.raw() < b.raw()) {
            best = Some((e.pos, far));
        }
    }
    match best {
        Some((pos, _)) => Quarry {
            pos,
            vel: V3::ZERO,
            stunned: false,
            ..real
        },
        None => real,
    }
}

fn spawn_the_only_one(effects: &mut [Option<Effect>; MAX_EFFECTS], effect: Effect) {
    for slot in effects.iter_mut() {
        if slot.is_some_and(|e| e.kind == effect.kind && e.owner == effect.owner) {
            *slot = None;
        }
    }
    spawn_effect(effects, effect);
}

/// **Is a body standing in this fire?** Every fire on the floor or in the
/// air, by the same volume it burns with: a pillar's two, a tornado's two
/// where it has got to, a cloud's ball, the carpet's strip and the
/// fountain's wash. What a Downdraft landing asks, and what a Thermal asks.
fn body_in_fire(e: &Effect, pos: V3, radius: Fx, height: Fx) -> bool {
    match e.kind {
        EffectKind::FirePillar => {
            let (base, column) = e.pillar_volumes();
            base.contains(e.pos, pos, radius, height) || column.contains(e.pos, pos, radius, height)
        }
        EffectKind::FireTornado => {
            let at = e.tornado_pos();
            let (base, column) = e.pillar_volumes();
            base.contains(at, pos, radius, height) || column.contains(at, pos, radius, height)
        }
        EffectKind::Embers => e.ember_volume().contains(e.pos, pos, radius, height),
        EffectKind::Fountain => e.fountain_volume().contains(e.pos, pos, radius, height),
        EffectKind::FireCarpet => {
            let (a, b) = e.carpet_line();
            let mid = V3::new(pos.x, pos.y.add(height.mul(Fx::ratio(1, 2))), pos.z);
            let body = V3::new(pos.x, pos.y.add(height), pos.z);
            crate::math::segment_gap(a, b, pos, body).raw() <= e.field_radius().add(radius).raw()
                || crate::math::segment_gap(a, b, mid, mid).raw()
                    <= e.field_radius().add(radius).raw()
        }
        _ => false,
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
    // **The Dual mage's feet leave the floor at the three-quarter tier**, and
    // a thing that is not walking is not held to a walk. See [`floating`] and
    // `tuning::float_move_speed`; the renderer reads the same predicate to stop
    // driving her legs -- `view::pose`.
    let speed = if floating(p) {
        speed.mul(t::float_move_speed())
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
    let slot = mount_slot(p.mount);
    let part = mount_part(p.mount);
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
    // **A cut is not a buck.** The pose is a lookup, so the frame the
    // creature changes clip -- a shake interrupted into a flinch, say -- its
    // back jumps from one pose to another with no motion in between, and the
    // grip test read that jump as an acceleration and threw whoever was
    // standing there. The rider who had just landed the hit that caused the
    // flinch was the one thrown, every time: every ride ended with the first
    // good hit on the ridge. Plant the feet again, as on landing, and let the
    // motion that follows be what is judged. Read here rather than at the top
    // of the frame because a flinch is dealt after the fighters have moved,
    // so the cut shows up under the feet a frame later than it happens.
    let clip = beast.doing.clip_key(beast.sp().moves.len());
    if p.ride_clip != clip {
        p.ride_clip = clip;
        p.grip_settle = p.grip_settle.max(t::mount_settle() as u8);
    }
    // **Inside it, nothing throws you**: there is nowhere to be thrown to.
    let inside = beast.sp().parts[part.min(beast.sp().parts.len() - 1)]
        .shape
        .hollow;
    if inside {
        p.grip_settle = t::mount_settle() as u8;
    }
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

    rider_acts(p, who, input, scene, out, pressed_mechanic);

    // Jumping is how you leave, and it carries the surface's own velocity with
    // you -- which is what makes stepping off the back of a charging animal a
    // real option rather than a mistake. Not from inside: the creature lets
    // you out, when it does.
    if input.has(Input::SPACE) && p.action.actionable() && !inside {
        let mob = p.class.mobility();
        p.mount = monster::NO_MOUNT;
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

    // Inside, the walls keep you in: the part's own box, less your width,
    // with your feet on its floor. No other part is a step or a wall here.
    if inside {
        let sh = beast.sp().shape(part);
        let lo = |v: Fx| v.add(radius);
        let hi = |v: Fx| v.sub(radius);
        next.x = next.x.clamp(lo(sh.min.x), hi(sh.max.x).max(lo(sh.min.x)));
        next.z = next.z.clamp(lo(sh.min.z), hi(sh.max.z).max(lo(sh.min.z)));
        next.y = sh.min.y;
        p.local = next;
        p.pos = rig.part_to_world(part, next);
        return;
    }

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
    let stepped = rig
        .surface_within(
            rig.part_to_world(part, next),
            radius,
            Fx::ZERO,
            t::step_up(),
        )
        .filter(|(up, _)| *up != part);
    if let Some((up, top)) = stepped {
        let mut rest = rig.world_to_part(up, rig.part_to_world(part, next));
        rest.y = top;
        p.mount = mount_of(slot, up);
        p.local = rest;
        p.pos = rig.part_to_world(up, rest);
        p.grip_settle = t::mount_settle() as u8;
        return;
    }

    let moved = rig.resolve(rig.part_to_world(part, next), radius, t::body_height());
    match rig.surface_under(moved.pos, radius) {
        Some((on, top)) => {
            // Staying on the same part keeps the body-space position that was
            // just computed rather than converting to the world and back. The
            // round trip is exact to about a millimetre, which is nothing once
            // and a crawl across the creature's back at sixty frames a second.
            let mut rest = if on == part && !moved.shoved {
                next
            } else {
                rig.world_to_part(on, moved.pos)
            };
            rest.y = top;
            if on != part {
                // Stepped from the barrel onto the tail, or the other way.
                p.grip_settle = t::mount_settle() as u8;
                p.mount = mount_of(slot, on);
            }
            p.local = rest;
            p.pos = rig.part_to_world(on, rest);
        }
        None => {
            // Walked off the edge. You leave with whatever the surface was
            // doing, which is why stepping off a turning animal throws you
            // wide -- capped the same way a jump is, and for the same reason.
            p.mount = monster::NO_MOUNT;
            p.pos = moved.pos;
            p.vel = carry_off(surface);
            p.grounded = false;
        }
    }
}

/// **What a rider does with their buttons**, on whatever they are riding --
/// a monster's part (`step_rider`) or a critter's back (`step_critter_rider`):
/// the mechanic, the chain, the shadow, a guard, and the moves, resolved
/// instead of the countdown exactly as on the ground. Shared so the two rides
/// cannot drift apart.
fn rider_acts(
    p: &mut Player,
    who: usize,
    input: Input,
    scene: &Scene,
    out: &[bool; moves::MAX_SLOTS],
    pressed_mechanic: bool,
) {
    step_mechanic(p, scene.arena);
    // Aboard, the Champion reads the standing row of its grid, so the chain is
    // live on the creature's back and cancels the same way it does on the
    // floor. There is no takeoff up here: jumping is how you *leave*, and a
    // move that spent the jump on an attack would take that away.
    chain_cancel(p);
    queue_the_shadow(p, input);
    // And the Reaver's strike on arrival, for a shadow left standing on the
    // creature's back -- the same cut as on the floor.
    shadow::swing_out_of_the_carry(p, input);
    let want_guard = input.has(Input::RIGHT) && p.shield().is_some_and(|sh| sh.in_hand());

    // Resolved instead of the countdown, exactly as on the ground -- see
    // `step_player`.
    if let Some((kind, held)) = p.action.channelling() {
        p.action = step_channel(p, who, kind, held, input, scene);
    } else {
        p.action = match countdown(p, want_guard, input) {
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
fn meet_the_creature(p: &mut Player, slot: usize, beast: &Monster) {
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
    // **A body that was just thrown does not land back on.** Mounting is
    // landing, and a rider in the stun of a buck is still being thrown; the
    // back that threw them is whipping about underneath, and a rolled top
    // face scooped a falling rider up a dozen frames after it had thrown
    // them, for the fall's damage a second time. The same goes for somebody
    // knocked off by a hit. They are solid to the animal still -- `resolve`
    // above -- so they slide off it rather than through it.
    if p.action.stunned() {
        return;
    }
    // Nor does a body. Since the creature learned to gallop and to charge, it
    // ran over the spot a fallen fighter lay on often enough to scoop them up
    // and carry them -- which the fight report counted as rides, and the game
    // drew as a corpse on its back.
    if p.health <= 0 {
        return;
    }
    let Some((part, top)) = beast.surface_under(p.pos, radius) else {
        return;
    };
    mount_on(p, slot, beast, part, top);
}

/// Take up station on a part. Mounting is landing: there is no button, because
/// a surface is a surface.
fn mount_on(p: &mut Player, slot: usize, beast: &Monster, part: usize, top: Fx) {
    p.ride_clip = beast.doing.clip_key(beast.sp().moves.len());
    let mut rest = beast.rest_frame(part, p.pos);
    rest.y = top;
    p.mount = mount_of(slot, part);
    p.local = rest;
    p.pos = beast.world_of(part, rest);
    p.vel = V3::ZERO;
    p.grounded = true;
    p.air_dodged = false;
    p.jump_hold = 0;
    p.air_stall = 0;
    p.air_stalls = 0;
    p.carpets = 0;
    p.grip_vel = V3::ZERO;
    p.grip_settle = t::mount_settle() as u8;
}

// ---------------------------------------------------------------------------
// Riding a critter
//
// The Hornback's cow (`docs/design/creatures/hornback.md` §10, item 2): a
// critter whose kind is `mountable` has a back -- the top of its box -- that a
// fighter lands on and rides, carried through the pitch and lift its species
// gives it (`PackMind::surface`, `Critter::back_point`). The rule is the
// monster ride's: **the position on the back is authoritative**, and the grip
// is tested against what the surface does under the feet, so a buck is
// acceleration rather than a flag. Smaller than `step_rider`: one box, no
// parts to step between, no clips to cut.
// ---------------------------------------------------------------------------

/// Land on a critter's back, if one is under a falling fighter.
fn meet_the_critters(p: &mut Player, crowd: &Critters) {
    if p.vel.y.raw() > 0 || p.action.stunned() || p.health <= 0 {
        return;
    }
    let sp = crowd.sp();
    for (i, c) in crowd.iter().enumerate() {
        if !c.rideable(sp) {
            continue;
        }
        let body = c.body(sp);
        let local = body.to_local(p.pos);
        // Any of the body over its back: a critter is narrow, and a person
        // jumps onto it from against its flank.
        let grace = t::body_radius();
        let on = local.x.abs().raw() <= body.half_len.add(grace).raw()
            && local.z.abs().raw() <= body.half_wid.add(grace).raw()
            && local.y.raw() <= body.height.raw()
            && local.y.raw() >= body.height.sub(t::mount_snap()).raw();
        if !on {
            continue;
        }
        p.mount = critter::mount_of(i);
        p.local = V3::new(local.x, Fx::ZERO, local.z);
        p.pos = c.back_point(sp, p.local);
        p.vel = V3::ZERO;
        p.grounded = true;
        p.air_dodged = false;
        p.jump_hold = 0;
        p.air_stall = 0;
        p.air_stalls = 0;
        p.carpets = 0;
        p.grip_vel = V3::ZERO;
        p.grip_settle = t::mount_settle() as u8;
        return;
    }
}

/// One tick of a fighter on a critter's back. See the section's notes.
fn step_critter_rider(
    p: &mut Player,
    who: usize,
    input: Input,
    c: &critter::Critter,
    sp: &'static crate::species::Species,
    scene: &Scene,
    out: &[bool; moves::MAX_SLOTS],
) {
    let held = p.local;
    let landing = c.back_point(sp, held);
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
    if p.action.actionable() || p.action.stunned() || p.action.channelling().is_some() {
        p.facing = look;
    } else if p.action.guarding() {
        p.facing = p
            .facing
            .add(look.sub(p.facing).scale(t::guard_turn_rate()))
            .normalized();
    }
    p.crouching = input.has(Input::CROUCH) && p.action.actionable();

    // **The grip.** The back dropping away from under the feet faster than
    // the grip holds -- the rump kicking up and stopping, the whole animal
    // lurching sideways -- throws the rider. Pressing up into the feet does
    // not: being shoved onto something is not being thrown off it.
    let felt = crate::math::big_len(V3::new(accel.x, accel.y.min(Fx::ZERO), accel.z));
    let grip = if p.crouching {
        t::grip().mul(t::brace_grip())
    } else {
        t::grip()
    };
    if p.grip_settle > 0 {
        p.grip_settle -= 1;
    } else if felt.raw() > grip.raw() {
        // Off the side they stand on, at the full kick, and up.
        let body = c.body(sp);
        let side = if held.z.raw() < 0 {
            Fx::ONE.neg()
        } else {
            Fx::ONE
        };
        let out_dir = body.dir_to_world(V3::new(Fx::ZERO, Fx::ZERO, side));
        p.mount = monster::NO_MOUNT;
        p.vel = V3::new(out_dir.x, Fx::ZERO, out_dir.z)
            .normalized()
            .scale(t::throw_kick())
            .add(V3::new(Fx::ZERO, t::throw_lift(), Fx::ZERO));
        p.grounded = false;
        p.air_dodged = false;
        p.jump_hold = 0;
        p.action = Action::HitStun {
            left: t::throw_stun(),
        };
        p.wound(t::throw_damage());
        return;
    }

    rider_acts(p, who, input, scene, out, pressed_mechanic);

    // Jumping is how you leave, with what the back was doing, capped.
    if input.has(Input::SPACE) && p.action.actionable() {
        let mob = p.class.mobility();
        p.mount = monster::NO_MOUNT;
        let carried = carry_off(surface);
        p.vel = carried.add(V3::new(Fx::ZERO, t::jump_speed().mul(mob.jump), Fx::ZERO));
        p.grounded = false;
        p.air_dodged = false;
        p.jump_hold = t::jump_hold_frames();
        return;
    }

    // Walking on its back, in its own frame; off the edge, a fall.
    let (ax, az) = input.move_axis();
    let speed = rider_speed(p);
    let body = c.body(sp);
    let wish = body.dir_to_local(move_dir(input.aim_turns(), ax, az));
    let next = V3::new(
        held.x.add(wish.x.mul(speed).mul(DT)),
        Fx::ZERO,
        held.z.add(wish.z.mul(speed).mul(DT)),
    );
    let walked = next.sub(held).flat_len();
    p.stride = p
        .stride
        .wrapping_add(walked.div(stride_length(p)).raw().clamp(0, 65535) as u16);
    let grace = t::body_radius();
    if next.x.abs().raw() > body.half_len.add(grace).raw()
        || next.z.abs().raw() > body.half_wid.add(grace).raw()
    {
        p.mount = monster::NO_MOUNT;
        p.pos = c.back_point(sp, next);
        p.vel = carry_off(surface);
        p.grounded = false;
        return;
    }
    p.local = next;
    p.pos = c.back_point(sp, next);
}

/// **Put a fighter inside a creature's hollow part** (`beast::Shape::hollow`):
/// its floor's middle, held in its own frame from here on, as a rider is. What
/// the Mireback's swallow does; the species lets them out with
/// [`let_out`].
pub fn put_inside(p: &mut Player, slot: usize, beast: &Monster, part: usize) {
    let sh = beast.sp().shape(part);
    let mid = sh.min.add(sh.max).scale(Fx::ratio(1, 2));
    let rest = V3::new(mid.x, sh.min.y, mid.z);
    p.ride_clip = beast.doing.clip_key(beast.sp().moves.len());
    p.mount = mount_of(slot, part);
    p.local = rest;
    p.pos = beast.world_of(part, rest);
    p.vel = V3::ZERO;
    p.grounded = true;
    p.air_dodged = false;
    p.jump_hold = 0;
    p.air_stall = 0;
    p.air_stalls = 0;
    p.carpets = 0;
    p.grip_vel = V3::ZERO;
    p.grip_settle = t::mount_settle() as u8;
}

/// **Let a fighter out of wherever they are inside**, at `at`, moving at
/// `vel`: no longer aboard, in the air.
pub fn let_out(p: &mut Player, at: V3, vel: V3) {
    p.mount = monster::NO_MOUNT;
    p.pos = at;
    p.vel = vel;
    p.grounded = false;
}

/// Is this fighter inside a creature, rather than on one?
pub fn inside(p: &Player, herd: &Herd) -> bool {
    ridden(p, herd).is_some_and(|beast| {
        let sp = beast.sp();
        sp.parts
            .get(mount_part(p.mount))
            .is_some_and(|part| part.shape.hollow)
    })
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
    // **Off the side, always.** The buck is the surface *accelerating*, and
    // the surface's velocity at that instant can be anything -- at the
    // reversal of a whip it is nearly nothing. The first version threw the
    // rider along the surface's velocity, capped, so a rider thrown at a
    // reversal went straight up seven metres, came straight back down onto
    // the same back, and was thrown again: five throws in a second, each
    // with the fall's damage on it. A rider comes off the side they were
    // standing on, at the full kick, whatever the back was doing.
    //
    // Sideways in the creature's **body** frame -- its heading, which does
    // not roll -- and up in the world's. The part underfoot is the thing that
    // is whipping: at the top of a shake the spine is fifty degrees over, and
    // "sideways" and "up" read in that frame are a throw that goes up and a
    // lift that goes across, which is a rider who lands back on the other
    // side of the same back.
    let side = if p.local.z.raw() < 0 {
        Fx::ONE.neg()
    } else {
        Fx::ONE
    };
    let out = rig.dir_to_world(V3::new(Fx::ZERO, Fx::ZERO, side));
    let flat = V3::new(out.x, Fx::ZERO, out.z);
    let _ = (surface, part);
    p.mount = monster::NO_MOUNT;
    p.vel =
        flat.normalized()
            .scale(t::throw_kick())
            .add(V3::new(Fx::ZERO, t::throw_lift(), Fx::ZERO));
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
fn fall_off(p: &mut Player, herd: &Herd) {
    if !p.aboard() {
        return;
    }
    if let Some(beast) = ridden(p, herd) {
        p.pos = beast.world_of(mount_part(p.mount), p.local);
    }
    p.mount = monster::NO_MOUNT;
    p.grounded = false;
}

/// The creature a rider is standing on, if it is still there.
fn ridden<'a>(p: &Player, herd: &'a Herd) -> Option<&'a Monster> {
    if !p.aboard() {
        return None;
    }
    herd.get(mount_slot(p.mount)).and_then(Option::as_ref)
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
    /// shot should still catch a stone that is kicked into the line during
    /// it. The move's one hit is what keeps that from being two shots --
    /// whatever the beam meets first ends it.
    ///
    /// **It meets only what she built, and walls.** Bodies are not on its
    /// list: it passes through fighters, creatures and critters with no
    /// damage and no interrupt. See `bolt::targets`.
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
        let m = moves::get(shooter.class, kind);
        // On through the point she aimed at, to the move's whole reach: what
        // is behind the stone the crosshair is on is on the line too.
        let beam = aim::through(beam_of(&shooter), m.reach);
        let field = stones::gather(&self.players);
        let seen = self.players;
        let effects = self.effects;
        let beast = self.monsters;
        let crowd = self.critters;
        let scene = Scene {
            stones: &field,
            players: &seen,
            effects: &effects,
            quarry: &beast,
            critters: &crowd,
            arena: &self.terrain(),
        };
        // **Everything on the line, at once** (2026-10-18): every stone and
        // every fire up to the first wall, so a stone in front of another
        // no longer takes the shot meant for the one behind it.
        let (met, n) = aim::every_along(beam, m.radius, i as u8, &scene, bolt::targets());

        // How long the line actually is, whether or not it still has a hit to
        // spend: to the wall that ends it, or its whole length. A spent beam
        // is still a beam, and it is still drawn.
        let (n, reach) = line_ends(&self.players, &met[..n], beam.length());
        self.players[i].beam_reach = reach;
        if shooter.hit_used {
            return;
        }

        for c in met[..n].iter().flatten() {
            match *c {
                // The stone goes along the line, pitch included: through the
                // ground aimed down it, up into the air aimed above it.
                Contact::Stone { index, .. } => {
                    if stones::is_lit(&self.players, index) {
                        self.burst_the_stone(i, index, beam.dir());
                    } else {
                        stones::kick(&mut self.players, index, beam.dir());
                    }
                    self.players[i].hit_used = true;
                }
                // A hazard is not a wall. The beam does not stop at the fire,
                // it lights it, and what leaves the pillar is the poke.
                Contact::Fire { dist, .. } => {
                    bolt::light(&mut self.bolts, i as u8, beam.at(dist), beam.dir());
                    self.players[i].hit_used = true;
                }
                // A solid wall: the line ends there, and it is the last
                // thing on the list.
                //
                // A body cannot arrive: the beam does not ask for one -- it
                // goes through people and acts only on what she built (see
                // `bolt::targets`). The arms are spelled out rather than
                // wildcarded so that whatever is added to `Contact` next is a
                // compile error here instead of silence.
                Contact::Terrain { .. }
                | Contact::Fighter { .. }
                | Contact::Quarry { .. }
                | Contact::Critter { .. } => {}
            }
        }
    }

    /// Fire Cataclysm, if this fighter has it out.
    ///
    /// Structurally the same trick the auto is: an instant line, resolved on
    /// the spot rather than by the hitbox loop, because what it does depends
    /// on what it meets first and none of the three answers is a bubble that
    /// lives for a few frames in front of her body. What it *does* on each
    /// answer is her heavy's own, not the auto's -- a structure is destroyed
    /// rather than kicked, and a fire pillar is torn loose into a travelling
    /// tornado rather than merely charging the shot. Like the auto it passes
    /// through bodies and stops at walls (`bolt::targets`): what hurts people
    /// is the debris, the burst and the tornado. See
    /// `docs/design/kits/elementalist.md`.
    fn fire_the_cataclysm(&mut self, i: usize) {
        let shooter = self.players[i];
        let Action::Active { kind, .. } = shooter.action else {
            return;
        };
        if !throws_a_cataclysm(&shooter, kind) || shooter.health <= 0 {
            return;
        }

        let m = moves::get(shooter.class, kind);
        // On through the point she aimed at, to the move's whole reach: what
        // is behind the stone the crosshair is on is on the line too.
        let beam = aim::through(beam_of(&shooter), m.reach);
        let field = stones::gather(&self.players);
        let seen = self.players;
        let effects = self.effects;
        let beast = self.monsters;
        let crowd = self.critters;
        let scene = Scene {
            stones: &field,
            players: &seen,
            effects: &effects,
            quarry: &beast,
            critters: &crowd,
            arena: &self.terrain(),
        };
        // Every stone and every fire on the line, as the beam's (2026-10-18).
        let (met, n) = aim::every_along(beam, m.radius, i as u8, &scene, bolt::targets());

        let (n, reach) = line_ends(&self.players, &met[..n], beam.length());
        self.players[i].beam_reach = reach;
        if shooter.hit_used {
            return;
        }

        for c in met[..n].iter().flatten() {
            match *c {
                // Broken outright, and thrown outward as debris rather than
                // detonated on the spot -- see `crate::debris`.
                Contact::Stone { index, .. } => {
                    if stones::is_lit(&self.players, index) {
                        self.burst_the_stone(i, index, beam.dir());
                    } else if let Some(at) = stones::destroy(&mut self.players, index) {
                        debris::blast(&mut self.debris, i as u8, at, beam.dir());
                    }
                    self.players[i].hit_used = true;
                }
                // Not charged -- torn loose. The same `Effect`, the same two
                // volumes, now moving. `age` and `life` are untouched: a
                // pillar that had barely started keeps growing on the same
                // clock it was already on, now while travelling, and one that
                // was already fully grown stays that way rather than snapping
                // back down to nothing and regrowing -- either would be a
                // visible glitch at the exact moment nothing about its *size*
                // actually changed. What is left of the original pillar's
                // life is what the tornado gets to travel on; the arena's own
                // edge is the other way it can run out. `banked` takes a
                // snapshot of `age` right here, so its flight can be measured
                // from this moment rather than from whenever the pillar was
                // first planted -- see `Effect::tornado_pos`. See
                // `crate::effects::EffectKind::FireTornado`.
                Contact::Fire { effect, .. } => {
                    if let Some(e) = self.effects[effect].as_mut() {
                        if e.kind == EffectKind::FirePillar {
                            e.kind = EffectKind::FireTornado;
                            // Flattened, not the raw beam: Cataclysm is only
                            // level when it was aimed at the ground (see
                            // `aim::skillshot_path`) -- anywhere else it
                            // meets a wall, a body height, or the range
                            // sphere at whatever pitch the camera happened to
                            // be at, and a tornado is a ground hazard the same
                            // way the pillar it came from was. A tilted `dir`
                            // walks `tornado_pos` straight through the floor
                            // (or into the sky) a couple of frames after it
                            // starts moving, and `arena::inside` reads that as
                            // having wandered off the map -- the tornado never
                            // gets to be seen at all, it just vanishes where
                            // the pillar stood.
                            e.dir = V3::new(beam.dir().x, Fx::ZERO, beam.dir().z).normalized();
                            e.struck.clear();
                            e.banked = e.age as i32;
                        }
                    }
                    self.players[i].hit_used = true;
                }
                // A solid wall ends the line; a body is never met. See the
                // same arm in `fire_the_beam`.
                Contact::Terrain { .. }
                | Contact::Fighter { .. }
                | Contact::Quarry { .. }
                | Contact::Critter { .. } => {}
            }
        }
    }
}

/// Where one of her two pushes ends: at the first wall, or the first planted
/// shield -- a Bulwark's wall is a wall to her line, as it always was -- or
/// its whole length; it goes through everything else (`aim::every_along`).
/// How many of `met` it reaches, and how long it is drawn.
fn line_ends(players: &[Player; MAX_PLAYERS], met: &[Option<Contact>], length: Fx) -> (usize, Fx) {
    for (k, c) in met.iter().enumerate() {
        let ends = match c {
            Some(Contact::Terrain { .. }) => true,
            Some(Contact::Stone { index, .. }) => matches!(
                players[index / class::MAX_STRUCTURES].mechanic,
                Mechanic::Shield(_)
            ),
            _ => false,
        };
        if ends {
            return (k + 1, c.map_or(length, |c| c.dist()));
        }
    }
    (met.len(), length)
}

/// Is this the Elementalist's heavy?
///
impl World {
    /// Fissure, on the move's first active frame: the stone she held churning
    /// **comes up while it slides** (2026-10-18, from play), along the line
    /// the hold bought, as a surge (`EffectKind::Surge`, stepped by
    /// [`World::step_the_surge`]).
    ///
    /// It used to be a crack raced out at once that stopped at the first body
    /// and erupted the stone there. Now the stone erupts from where it was
    /// held and travels: whoever stands at its front is carried -- her too,
    /// so held at her own feet it is a ride she jumps off at its speed -- and
    /// everybody else it reaches is struck once and shoved on, harder and
    /// further the longer the hold. The line is still left rough.
    ///
    /// Without a held stone -- the slot thrown some other way, or the stone
    /// evicted mid-hold -- the surge still runs from her feet and still
    /// strikes; it only has no stone to carry.
    fn race_the_crack(&mut self, i: usize, input: Input) {
        let p = self.players[i];
        let m = moves::get(p.class, SLOT_COMMITTED);
        let field = stones::gather(&self.players);
        let seen = self.players;
        let effects = self.effects;
        let beast = self.monsters;
        let crowd = self.critters;
        let scene = Scene {
            stones: &field,
            players: &seen,
            effects: &effects,
            quarry: &beast,
            critters: &crowd,
            arena: &self.terrain(),
        };
        // Along the floor it was raised on: stopped at an edge rather than
        // run off it into the floor below (`aim::kept_along`), and at a wall.
        let path = aim::kept_along(p.aim_path, &scene);
        // Walls asked a step up off the floor it runs on (2026-10-21): along
        // the floor itself the line grazes the top of the box it was raised
        // on, which read as a wall at no distance, and a crack held on any
        // raised floor never went anywhere. A step she would walk up is not a
        // wall to it either.
        let lift = V3::new(Fx::ZERO, t::step_up(), Fx::ZERO);
        let probe = aim::Path {
            from: path.from.add(lift),
            to: path.to.add(lift),
        };
        let length = aim::first_along(probe, Fx::ZERO, i as u8, &scene, Targets::none().walls())
            .map_or(path.length(), |c| c.dist());
        let dir = path.dir();
        let flat = V3::new(dir.x, Fx::ZERO, dir.z).normalized();
        // How much of a full hold this was: a tap's run is the row's
        // `channel, reach at no hold`, a full one its `reach`.
        let least = m.channel_from;
        let share = if m.reach.raw() > least.raw() {
            p.aim_path
                .length()
                .sub(least)
                .div(m.reach.sub(least))
                .max(Fx::ZERO)
                .min(Fx::ONE)
        } else {
            Fx::ONE
        };
        let held = self.players[i].charging_stone;
        let stone = (held != NO_STONE).then(|| i * class::MAX_STRUCTURES + held as usize);
        // The stone erupts where it was held, and the surge carries it on.
        if let Some(index) = stone {
            let ground = self.terrain();
            stones::relocate_and_erupt(&mut self.players, index, path.from, &ground);
            self.players[i].charging_stone = NO_STONE;
        }
        spawn_effect(
            &mut self.effects,
            Effect::surge(
                i as u8,
                p.class,
                SLOT_COMMITTED,
                path.from,
                dir,
                length,
                stone,
                share,
            ),
        );
        // And the scar: the whole line it will run.
        if length.raw() > 0 {
            let from = V3::new(
                path.from.x,
                self.terrain().ground_under(path.from),
                path.from.z,
            );
            spawn_effect(
                &mut self.effects,
                Effect::cast(
                    EffectKind::Rough,
                    i as u8,
                    p.class,
                    SLOT_COMMITTED,
                    from,
                    flat,
                    length,
                ),
            );
        }
        self.players[i].hit_used = true;
        // `input` is what the crack was aimed with; the path was locked on
        // release and this only reads it, but the frame's input is what every
        // other first-active-frame hand-off receives, and the signature says
        // so for the day the crack wants the stick.
        let _ = input;
    }

    /// **One frame of Fissure's surge**: the front moves on along its line,
    /// carrying its stone -- coming up out of the ground on its own clock --
    /// and every body standing at it; anybody but her it reaches for the
    /// first time is struck, and thrown on along the line.
    ///
    /// **Carried** is a moving floor, as the Air ball's is: a body at the
    /// front on its feet is moved with it frame for frame, and one off its
    /// feet -- jumped, or thrown -- is given its speed along the line, so
    /// jumping out of the eruption at the right moment leaves with both the
    /// stone's climb and the surge's run.
    fn step_the_surge(&mut self, effect: &mut Effect) {
        let owner = effect.owner as usize;
        let front = effect.surge_front();
        let was = effect.surge_front_at(effect.age.saturating_sub(1));
        let moved = V3::new(front.x.sub(was.x), Fx::ZERO, front.z.sub(was.z));
        let flat = V3::new(effect.dir.x, Fx::ZERO, effect.dir.z).normalized();
        let running = moved.flat_len().raw() > 0;
        let speed = t::surge_speed();
        let ground = self.terrain();
        if let Some(index) = (effect.banked as usize).checked_sub(1) {
            if running {
                stones::slide_to(&mut self.players, index, front, &ground);
            }
            // **Off the end of her level, over the edge** (2026-10-21, from
            // play): the run stops at a lip (`aim::kept_along`), and a stone
            // that got there is let go at the surge's speed rather than left
            // standing on it, so it goes on over and falls to the floor below.
            // A run that ended at a wall, or on more floor, keeps its stone.
            let end = effect.pos.add(effect.dir.scale(effect.reach));
            // Its last frame, too: its life is the run rounded to frames, and
            // can end a hair short of the end.
            if front == end || effect.age.saturating_add(1) >= effect.life {
                let past = front.add(flat.scale(t::body_radius()));
                let below = ground.floor_below(V3::new(past.x, front.y.add(arena::SKIN), past.z));
                if below.raw() < front.y.sub(t::placement_drop()).raw() {
                    stones::shove(&mut self.players, index, flat, speed);
                    effect.banked = 0;
                }
            }
        }
        let m = effect.source();
        let reach = m.radius.add(t::body_radius());
        let top = front.y.add(t::structure_height());
        let power = effect.power;
        let worth = |gain: Fx| Fx::ONE.add(gain.mul(power));
        for i in 0..MAX_PLAYERS {
            let q = self.players[i];
            if q.health <= 0 {
                continue;
            }
            let apart = V3::new(q.pos.x.sub(front.x), Fx::ZERO, q.pos.z.sub(front.z));
            let at_front = apart.flat_len().raw() <= reach.raw()
                && q.pos.y.raw() >= front.y.sub(t::body_radius()).raw()
                && q.pos.y.raw() <= top.add(t::body_radius()).raw();
            if !at_front {
                continue;
            }
            // Struck, the first time it reaches anybody but her.
            if i != owner && self.effects_reach(i, effect.owner) && effect.take_hit(0, i) {
                let (guarding, parried) = guard_against(&q, front, m.unblockable);
                apply_hit(
                    &mut self.players[i],
                    Hit {
                        damage: Fx::from_int(m.damage)
                            .mul(worth(t::surge_damage_gain()))
                            .to_int(),
                        hitstun: m.hitstun,
                        blockstun: m.blockstun,
                        knockback: m.knockback.mul(worth(t::surge_knock_gain())),
                        launch: t::surge_lift().mul(worth(t::surge_knock_gain())),
                        grabs: 0,
                        by: effect.owner,
                        dir: flat,
                        blocked: guarding,
                        parried,
                        interrupts: true,
                    },
                );
                if parried {
                    self.players[i].parried = PARRY_FLOURISH;
                }
                continue;
            }
            if !running {
                continue;
            }
            // Carried: on its feet, moved with it; off them, given its run.
            let r = &mut self.players[i];
            if r.grounded {
                r.pos.x = r.pos.x.add(moved.x);
                r.pos.z = r.pos.z.add(moved.z);
            } else {
                let along = r.vel.x.mul(flat.x).add(r.vel.z.mul(flat.z));
                if along.raw() < speed.raw() {
                    let more = speed.sub(along);
                    r.vel.x = r.vel.x.add(flat.x.mul(more));
                    r.vel.z = r.vel.z.add(flat.z.mul(more));
                }
            }
        }
        // The creatures and the small bodies it reaches, once each, at the
        // hold's worth.
        let share = worth(t::surge_damage_gain());
        for slot in 0..MAX_MONSTERS {
            if self.monsters[slot].is_some() {
                self.gore_for_a_share(slot, effect, 0, front, m.radius, share);
            }
        }
        self.gore_critters_by(effect, EffectReach::Column(front, m.radius), share);
    }

    /// The shake is over: everyone still in the patch takes the eruption and
    /// is shoved outward, and a stone comes up at its centre for the caster
    /// -- under her own feet, for a Tremor, which is what lifts her.
    fn erupt_the_quake(&mut self, effect: &Effect) {
        let radius = effect.field_radius().add(t::body_radius());
        for i in 0..MAX_PLAYERS {
            let p = self.players[i];
            let apart = V3::new(
                p.pos.x.sub(effect.pos.x),
                Fx::ZERO,
                p.pos.z.sub(effect.pos.z),
            );
            if !self.effects_reach(i, effect.owner) || apart.flat_len().raw() > radius.raw() {
                continue;
            }
            let away = if apart.flat_len().raw() > 0 {
                apart.normalized()
            } else {
                self.players[effect.owner as usize].facing
            };
            let (guarding, parried) = guard_against(&p, effect.pos, false);
            apply_hit(
                &mut self.players[i],
                Hit {
                    damage: effect.damage(),
                    hitstun: t::stone_erupt_stagger(),
                    blockstun: t::stone_erupt_stagger(),
                    knockback: t::quake_push(),
                    launch: Fx::ZERO,
                    grabs: 0,
                    by: effect.owner,
                    dir: away,
                    blocked: guarding,
                    parried,
                    interrupts: true,
                },
            );
            if parried {
                self.players[i].parried = PARRY_FLOURISH;
            }
        }
        // The small bodies standing in it are thrown about with everybody
        // else: the Gnawers' doc has the Quake emptying a horseshoe of them.
        let mut erupting = *effect;
        self.gore_critters(&mut erupting, effect.pos, effect.field_radius(), Fx::ONE);
        let owner = effect.owner as usize;
        if owner < MAX_PLAYERS {
            let field = stones::gather(&self.players);
            let at = aim::settle(effect.pos, &field, &self.terrain());
            stones::raise(&mut self.players[owner], class::Structure::raised(at));
        }
    }

    /// The Elementalist's dodge carried her into one of her stones: it breaks
    /// down as she passes, and where it stood is rough terrain -- burning
    /// ground, with a cloud of embers, if the stone was lit.
    ///
    /// The scar runs across the stone's footprint along the dodge, so the
    /// pursuer who follows her line crosses the whole of it.
    fn break_through(&mut self, i: usize, slot: usize) {
        let p = self.players[i];
        let index = i * class::MAX_STRUCTURES + slot;
        let lit = stones::is_lit(&self.players, index);
        let Some(middle) = stones::destroy(&mut self.players, index) else {
            return;
        };
        let along = V3::new(p.vel.x, Fx::ZERO, p.vel.z);
        let dir = if along.flat_len().raw() > 0 {
            along.normalized()
        } else {
            p.facing
        };
        let across = t::structure_radius().mul(Fx::from_int(2));
        let from = V3::new(
            middle.x.sub(dir.x.mul(t::structure_radius())),
            self.terrain().ground_under(middle),
            middle.z.sub(dir.z.mul(t::structure_radius())),
        );
        spawn_effect(
            &mut self.effects,
            Effect::cast(
                EffectKind::Rough,
                i as u8,
                p.class,
                SLOT_COMMITTED,
                from,
                dir,
                across,
            ),
        );
        if lit {
            spawn_effect(
                &mut self.effects,
                Effect::cast(
                    EffectKind::Embers,
                    i as u8,
                    p.class,
                    moves::elementalist::CINDER,
                    middle,
                    V3::ZERO,
                    t::embers_radius(),
                ),
            );
        }
    }

    /// A lit stone, shoved or broken: it goes, its pieces fly along `dir` as
    /// burning debris, and a cloud of embers is left where it stood.
    fn burst_the_stone(&mut self, owner: usize, index: usize, dir: V3) {
        let Some(middle) = stones::destroy(&mut self.players, index) else {
            return;
        };
        debris::blast(&mut self.debris, owner as u8, middle, dir);
        let class = self.players[owner].class;
        spawn_effect(
            &mut self.effects,
            Effect::cast(
                EffectKind::Embers,
                owner as u8,
                class,
                moves::elementalist::CINDER,
                middle,
                V3::ZERO,
                t::embers_radius(),
            ),
        );
    }

    /// **Fire on a stone lights it, every time they touch** (2026-10-18):
    /// every fire standing in the world this frame, in its own shape, against
    /// every stone. The pillar's base and column, a tornado's where it has
    /// got to, a cloud of embers, the fountain's wash, the carpet's strip and
    /// the ring of fire out to how far it has raced. The fire bolt lights the
    /// stone that stops it (`bolt::step`) and a lit shot the stones it passes
    /// (`gust::step`), each where it already meets them.
    fn fire_lights_the_stones(&mut self) {
        if !stones::gather(&self.players).iter().any(Option::is_some) {
            return;
        }
        let fire = self.effects;
        for e in fire.iter().flatten() {
            if !e.kind.ignites() {
                continue;
            }
            match e.kind {
                EffectKind::FirePillar | EffectKind::FireTornado => {
                    let at = if e.kind == EffectKind::FireTornado {
                        e.tornado_pos()
                    } else {
                        e.pos
                    };
                    let (base, column) = e.pillar_volumes();
                    for slab in [base, column] {
                        let foot = V3::new(at.x, at.y.add(slab.bottom), at.z);
                        stones::light_touching(
                            &mut self.players,
                            foot,
                            slab.radius,
                            slab.top.sub(slab.bottom),
                        );
                    }
                }
                EffectKind::FireCarpet => {
                    let (a, b) = e.carpet_line();
                    stones::light_along(&mut self.players, a, b, e.field_radius());
                }
                EffectKind::FireRing => {
                    stones::light_touching(
                        &mut self.players,
                        e.pos,
                        e.ring_radius(),
                        t::body_height(),
                    );
                }
                _ => {
                    let slab = if e.kind == EffectKind::Fountain {
                        e.fountain_volume()
                    } else {
                        e.ember_volume()
                    };
                    let foot = V3::new(e.pos.x, e.pos.y.add(slab.bottom), e.pos.z);
                    stones::light_touching(
                        &mut self.players,
                        foot,
                        slab.radius,
                        slab.top.sub(slab.bottom),
                    );
                }
            }
        }
    }

    /// Put a draft's column under the Elementalist's feet.
    fn raise_the_draft(&mut self, i: usize, kind: EffectKind) {
        let p = self.players[i];
        spawn_effect(
            &mut self.effects,
            Effect::cast(
                kind,
                i as u8,
                p.class,
                match kind {
                    EffectKind::Updraft => moves::elementalist::UPDRAFT,
                    _ => moves::elementalist::DOWNDRAFT,
                },
                p.pos,
                V3::ZERO,
                t::draft_radius(),
            ),
        );
    }

    /// The Elementalist has just landed. If a Downdraft of hers is still
    /// blowing, it bursts: over plain ground the air breaks outward as a ring
    /// that shoves everyone nearby; into fire -- a pillar, a tornado or a
    /// cloud of embers she landed in -- the fire goes out and a ring of fire
    /// races outward instead. Either way the column is spent.
    ///
    /// The fire it consumes is whatever she is standing in, by the same
    /// tests those effects use on a body. One ring per landing, whatever
    /// burned there; what it is worth does not stack.
    fn land_the_draft(&mut self, i: usize) {
        let p = self.players[i];
        if p.class != Class::Elementalist {
            return;
        }
        let Some(draft) = self
            .effects
            .iter()
            .position(|e| e.is_some_and(|e| e.kind == EffectKind::Downdraft && e.owner == i as u8))
        else {
            return;
        };
        self.effects[draft] = None;
        let (radius, height) = (t::body_radius(), t::body_height());
        let mut burned = false;
        for slot in 0..MAX_EFFECTS {
            let Some(e) = self.effects[slot] else {
                continue;
            };
            if body_in_fire(&e, p.pos, radius, height) {
                self.effects[slot] = None;
                burned = true;
            }
        }
        let kind = if burned {
            EffectKind::FireRing
        } else {
            EffectKind::AirRing
        };
        let reach = if burned {
            t::fire_ring_reach()
        } else {
            t::air_ring_radius()
        };
        spawn_effect(
            &mut self.effects,
            Effect::cast(
                kind,
                i as u8,
                p.class,
                moves::elementalist::DOWNDRAFT,
                p.pos,
                V3::ZERO,
                reach,
            ),
        );
    }

    /// **The Blood mage's moves, on the frame each comes out** (2026-10-12):
    /// what each puts in the world besides its row's own volume or effect.
    /// The Thorn lash, the spike and the Bramble are their rows' effects,
    /// thrown in `advance` with the rest; the Sweep and the Reap are swings.
    fn blood_comes_out(&mut self, i: usize, kind: u8) {
        use moves::blood as b;
        match kind {
            b::THORNFALL => self.thornfall(i),
            b::STAKE => self.throw_the_stake(i),
            b::COAGULATE => self.coagulate(i),
            b::RISING_REAP => self.rising_reap(i),
            b::HEART => self.plant_a_heart(i),
            b::HARVEST => self.harvest(i),
            // The geyser is the row's own disc where it was aimed; the floor
            // under it is hers.
            b::UPHEAVAL => {
                let at = self.floor_at_aim(i);
                let radius = moves::get(Class::BloodMage, b::UPHEAVAL).radius;
                self.lay_touch(i, at, radius);
            }
            // The vines were made from the field the moment the hold ran out
            // (`step_plagues`); the cascade goes out on its active frame.
            b::CASCADE => self.start_the_cascade(i),
            _ => {}
        }
    }

    /// **Thornfall** (2026-10-18; Blood wings before it): a burst along her
    /// aim, any way at all, stopped at the first wall or stone. Where it ends
    /// she lands in a ring of thorns (`land_the_thornfall`).
    fn thornfall(&mut self, i: usize) {
        let p = self.players[i];
        let m = moves::get(p.class, moves::blood::THORNFALL);
        let stones = stones::gather(&self.players);
        let seen = self.players;
        let effects = self.effects;
        let scene = aim::Scene {
            stones: &stones,
            players: &seen,
            effects: &effects,
            quarry: &self.monsters,
            critters: &self.critters,
            arena: &self.terrain(),
        };
        let dir = p.aim_path.dir();
        let from = aim::origin(p.pos);
        let path = aim::Path {
            from,
            to: from.add(dir.scale(m.reach)),
        };
        let reach = match aim::first_along(
            path,
            t::body_radius(),
            i as u8,
            &scene,
            aim::Targets::none().terrain().stones(),
        ) {
            Some(c) => c.dist().sub(t::body_radius()),
            None => m.reach,
        };
        if reach.raw() <= 0 {
            self.ring_of_thorns(i, p.pos);
            return;
        }
        let frames = t::wings_frames();
        let speed = reach.div(Fx::from_int(frames as i32).mul(DT));
        begin_haul(
            &mut self.players[i],
            p.pos.add(dir.scale(reach)),
            speed,
            frames + 1,
        );
        self.players[i].grounded = false;
    }

    /// **Thornfall lands**: every Blood mage whose Thornfall haul ended this
    /// frame, still in the move, bursts into a ring of thorns where she is.
    /// `seen` is the fighters as the frame found them.
    fn land_the_thornfall(&mut self, seen: &[Player; MAX_PLAYERS]) {
        for (i, was) in seen.iter().enumerate() {
            let now = self.players[i];
            if now.class == Class::BloodMage
                && was.haul > 0
                && now.haul == 0
                && now.action.attack_kind() == Some(moves::blood::THORNFALL)
            {
                self.ring_of_thorns(i, now.pos);
            }
        }
    }

    /// A ring of thorns at `at`, Thornfall's row's size and blow, and her
    /// ground under it (its first frame, `touches_the_floor`).
    fn ring_of_thorns(&mut self, i: usize, at: V3) {
        let p = self.players[i];
        let m = moves::get(p.class, moves::blood::THORNFALL);
        let mut ring = Effect::cast(
            EffectKind::Nova,
            i as u8,
            p.class,
            moves::blood::THORNFALL,
            at,
            V3::ZERO,
            m.radius,
        );
        ring.power = depth(&p);
        spawn_effect(&mut self.effects, ring);
    }

    /// **The Thorn lash lands**: at the end of its flight -- the body it
    /// struck, or as far as it went -- it lays her ground on the floor under
    /// it, hit or miss, if the floor is near enough to have been reached.
    fn land_the_lash(&mut self, effect: &Effect) {
        if effect.kind != EffectKind::Haemorrhage
            || effect.class != Class::BloodMage
            || effect.owner as usize >= MAX_PLAYERS
        {
            return;
        }
        let at = effect.bolt_at();
        let floor = self.terrain().ground_under(at);
        if at.y.sub(floor).raw() > t::body_height().raw() {
            return;
        }
        let radius = effect.field_radius();
        self.lay_touch(effect.owner as usize, V3::new(at.x, floor, at.z), radius);
    }

    /// **Lay her ground where a move of hers touched** (2026-10-18): a patch
    /// `radius` wide (never less than `tuning::ground_touch_least`) for
    /// `ground_touch_lasts`. **Thrown off her ground, it seeds more**: the
    /// weak version of a move leaves `ground_seed_off` as much behind it,
    /// wide and long. Which version it was is what she started her last move
    /// on (`Player::thrown_at`).
    fn lay_touch(&mut self, owner: usize, at: V3, radius: Fx) {
        let Some(p) = self.players.get(owner) else {
            return;
        };
        if p.class != Class::BloodMage {
            return;
        }
        let radius = radius.max(t::ground_touch_least());
        let life = t::ground_touch_lasts();
        let (radius, life) = if p.thrown_at.raw() > Fx::ONE.raw() {
            (radius, life)
        } else {
            let more = t::ground_seed_off();
            (
                radius.mul(more),
                Fx::from_int(life as i32)
                    .mul(more)
                    .to_int()
                    .clamp(1, u16::MAX as i32) as u16,
            )
        };
        self.lay_ground(owner as u8, at, radius, life);
    }

    /// Every disc of ground that is not a patch, of every owner's: her stains
    /// standing (`Effect::stain_ground`) and her hearts
    /// (`tuning::heart_ground`). With how many there are.
    fn blood_discs(&self) -> ([crate::ground::Disc; MAX_EFFECTS], usize) {
        let mut discs = [crate::ground::Disc::default(); MAX_EFFECTS];
        let mut n = 0;
        for e in self.effects.iter().flatten() {
            if e.class != Class::BloodMage || e.owner as usize >= MAX_PLAYERS {
                continue;
            }
            let radius = if e.is_a_pool() {
                e.stain_ground()
            } else if e.is_a_heart() {
                t::heart_ground()
            } else {
                continue;
            };
            discs[n] = crate::ground::Disc {
                at: e.pos,
                radius,
                owner: e.owner,
                holds: e.is_a_heart(),
            };
            n += 1;
        }
        (discs, n)
    }

    /// Is `at` on `owner`'s ground -- a patch, a stain or a heart?
    pub fn on_blood(&self, owner: usize, at: V3) -> bool {
        let (discs, n) = self.blood_discs();
        crate::ground::covers(&self.ground, &discs[..n], owner as u8, at)
    }

    /// Is `at` within `by` of `owner`'s ground, flat?
    fn near_blood(&self, owner: usize, at: V3, by: Fx) -> bool {
        let (discs, n) = self.blood_discs();
        crate::ground::near(&self.ground, &discs[..n], owner as u8, at, by)
    }

    /// The middle of `owner`'s ground nearest `at`, if she has any.
    fn nearest_blood(&self, owner: usize, at: V3) -> Option<V3> {
        let (discs, n) = self.blood_discs();
        crate::ground::nearest(&self.ground, &discs[..n], owner as u8, at)
    }

    /// Lay a patch of `owner`'s ground (`crate::ground::lay_sparing`), with
    /// what her hearts hold spared and, at the cap, the held patch furthest
    /// from them making way.
    fn lay_ground(&mut self, owner: u8, at: V3, radius: Fx, life: u16) {
        let (discs, n) = self.blood_discs();
        let held = self.ground_held;
        crate::ground::lay_sparing(&mut self.ground, held, &discs[..n], owner, at, radius, life);
    }

    /// **The ground, one frame** (`crate::ground::step`).
    fn step_ground(&mut self) {
        let (discs, n) = self.blood_discs();
        self.ground_held = crate::ground::step(&mut self.ground, &discs[..n], self.frame);
    }

    /// **Who is on her own ground** (2026-10-18), once a frame before anybody
    /// moves: every Blood mage on the floor (or swimming) inside a patch, a
    /// stain or a heart of hers. Everybody else is not. A finisher's fuel
    /// that has outlived its finisher -- the Harvest's, which is spent on its
    /// swing rather than on the frame it comes out -- is let go here.
    fn mark_the_blood(&mut self) {
        let (discs, n) = self.blood_discs();
        for i in 0..MAX_PLAYERS {
            let p = self.players[i];
            let on = p.class == Class::BloodMage
                && p.health > 0
                && (p.grounded || p.swimming())
                && crate::ground::covers(&self.ground, &discs[..n], i as u8, p.pos);
            self.players[i].on_blood = on;
            if p.fuel > 0 && !p.action.attack_kind().is_some_and(moves::blood::finisher) {
                self.players[i].fuel = 0;
            }
        }
    }

    /// **Plant a Heart** (2026-10-18), at the floor under the crosshair: it
    /// holds what she paid for it, `tuning::heart_from_paid` times over and
    /// never less than `heart_least`, and that is the blood blows take off it
    /// and thralls are made of. What she paid goes into it rather than onto
    /// the floor as a stain. Past `hearts_most` her oldest gives way.
    fn plant_a_heart(&mut self, i: usize) {
        let p = self.players[i];
        let at = self.floor_at_aim(i);
        let paid = p.paid as i32;
        self.players[i].paid = 0;
        self.players[i].heart_meter = 0;
        let blood = Fx::from_int(paid)
            .mul(t::heart_from_paid())
            .to_int()
            .max(t::heart_least());
        let mut hearts = 0;
        let mut oldest: Option<(usize, u16)> = None;
        for (slot, e) in self.effects.iter().enumerate() {
            let Some(e) = e else { continue };
            if e.is_a_heart() && e.owner as usize == i {
                hearts += 1;
                if oldest.is_none_or(|(_, age)| e.age > age) {
                    oldest = Some((slot, e.age));
                }
            }
        }
        if hearts >= t::hearts_most()
            && let Some((slot, _)) = oldest
        {
            self.effects[slot] = None;
        }
        let mut heart = Effect::cast(
            EffectKind::Heart,
            i as u8,
            p.class,
            moves::blood::HEART,
            at,
            V3::ZERO,
            t::heart_ground(),
        );
        heart.banked = blood;
        spawn_effect(&mut self.effects, heart);
    }

    /// **Every Heart, one frame.** Blows out on it come off its blood (the
    /// same blows a thrall feels, `blow_on_a_thrall`, a body's size); at
    /// nothing it is gone. Every `tuning::heart_every` frames, while she has
    /// room for another thrall and it has the blood, it spends
    /// `heart_thrall` of it on one at its foot.
    fn step_hearts(&mut self) {
        for slot in 0..MAX_EFFECTS {
            let Some(mut heart) = self.effects[slot].filter(|e| e.is_a_heart()) else {
                continue;
            };
            let owner = heart.owner as usize;
            let blow = self.blow_on_a_thrall(&heart, t::body_radius(), t::body_height());
            if blow > 0 && !heart.already_hit(0, 0) {
                heart.banked -= blow;
                heart.take_hit(0, 0);
            } else if blow == 0 {
                heart.forget_hits();
            }
            if heart.banked <= 0 || owner >= MAX_PLAYERS {
                self.effects[slot] = None;
                continue;
            }
            let every = t::heart_every().max(1);
            // It stands as long as nobody beats it down, so its clock is
            // held under the top of its counter, a whole beat at a time, so
            // its thralls keep their rhythm and it stays the oldest.
            if heart.age > u16::MAX - 2 * every {
                heart.age -= every;
            }
            let cost = t::heart_thrall();
            if (heart.age + 1) % every == 0
                && heart.banked > cost
                && self.thralls_of(owner) < t::thrall_cap()
            {
                // Out at its foot, on a side that turns with every one.
                let turn = Fx::ratio((heart.age / every) as i32 % 6, 6);
                let off = V3::new(
                    crate::fixed::cos_turns(turn),
                    Fx::ZERO,
                    crate::fixed::sin_turns(turn),
                )
                .scale(t::body_radius().add(t::body_radius()));
                let at = heart.pos.add(off);
                let lift = V3::new(at.x, heart.pos.y.add(Fx::ONE), at.z);
                let at = V3::new(at.x, self.terrain().ground_under(lift), at.z);
                let thrall = Effect::pool(heart.owner, heart.class, moves::blood::HEART, at, cost);
                if let Some(born) = spawn_effect_at(&mut self.effects, thrall) {
                    self.stand_up(born);
                    heart.banked -= cost;
                }
            }
            self.effects[slot] = Some(heart);
        }
    }

    /// **The Harvest** (2026-10-18; the golem's row): the finisher of the
    /// Reap. Its blow is the great turning swing itself; what comes out
    /// with it is that every stain of hers within `tuning::harvest_reach`
    /// is drunk, as the blade goes round through them.
    fn harvest(&mut self, i: usize) {
        let p = self.players[i];
        let m = moves::get(p.class, moves::blood::HARVEST);
        for slot in self.stains_within(i, p.pos, t::harvest_reach()) {
            self.drink_from(slot, i, &m);
        }
    }

    /// **Coagulate**: every stain of hers within `tuning::coagulate_reach` of
    /// the crosshair's spot on the floor slides there (`slide_the_stains`)
    /// and they merge into one.
    fn coagulate(&mut self, i: usize) {
        let p = self.players[i];
        let at = p.aim_at();
        let lift = V3::new(at.x, at.y.add(Fx::ONE), at.z);
        let to = V3::new(at.x, self.terrain().ground_under(lift), at.z);
        for e in self.effects.iter_mut().flatten() {
            if e.is_a_pool()
                && e.owner == i as u8
                && V3::new(e.pos.x.sub(to.x), Fx::ZERO, e.pos.z.sub(to.z))
                    .flat_len()
                    .raw()
                    <= t::coagulate_reach().raw()
            {
                e.home = to;
            }
        }
    }

    /// **Stains being gathered slide**, at `tuning::coagulate_speed`, to where
    /// they were sent, and two of one owner's that meet merge into one. A
    /// stain's `home` is where it was spilled until Coagulate moves it.
    fn slide_the_stains(&mut self) {
        let step = t::coagulate_speed().mul(DT);
        for slot in 0..MAX_EFFECTS {
            let Some(mut e) = self.effects[slot].filter(|e| e.is_a_pool()) else {
                continue;
            };
            let gap = V3::new(e.home.x.sub(e.pos.x), Fx::ZERO, e.home.z.sub(e.pos.z));
            let far = gap.flat_len();
            if far.raw() == 0 {
                continue;
            }
            if far.raw() <= step.raw() {
                e.pos = e.home;
            } else {
                let moved = e.pos.add(gap.normalized().scale(step));
                let lift = V3::new(moved.x, moved.y.add(Fx::ONE), moved.z);
                e.pos = V3::new(moved.x, self.terrain().ground_under(lift), moved.z);
            }
            self.effects[slot] = Some(e);
            // Merged into any of hers it has reached.
            for other in 0..MAX_EFFECTS {
                if other == slot {
                    continue;
                }
                let Some(o) = self.effects[other].filter(|o| o.is_a_pool() && o.owner == e.owner)
                else {
                    continue;
                };
                let apart =
                    V3::new(o.pos.x.sub(e.pos.x), Fx::ZERO, o.pos.z.sub(e.pos.z)).flat_len();
                if apart.raw() <= e.pool_radius().add(o.pool_radius()).raw() {
                    if let Some(keep) = self.effects[slot].as_mut() {
                        keep.banked += o.banked;
                    }
                    self.effects[other] = None;
                }
            }
        }
    }

    /// **Rising reap**: a sweep up and out in the cone she faces,
    /// `move.reach` long. It hurls her thralls in it along it, drinks her
    /// stains in it, and is a light hit on whoever is in it -- which bleeds,
    /// as every blow does.
    fn rising_reap(&mut self, i: usize) {
        let p = self.players[i];
        let m = moves::get(p.class, moves::blood::RISING_REAP);
        let look = p.aim_path.dir();
        let dir = V3::new(look.x, Fx::ZERO, look.z).normalized();
        let in_cone = |at: V3| {
            let off = V3::new(at.x.sub(p.pos.x), Fx::ZERO, at.z.sub(p.pos.z));
            let far = off.flat_len();
            far.raw() <= m.reach.raw()
                && (far.raw() <= t::body_radius().raw()
                    || off.dot(dir).raw() >= far.mul(t::reap_cone()).raw())
        };
        let mut drunk = 0;
        for slot in 0..MAX_EFFECTS {
            let Some(mut e) = self.effects[slot] else {
                continue;
            };
            if e.owner != i as u8 || !in_cone(e.pos) {
                continue;
            }
            if e.is_a_thrall() {
                e.dir = dir;
                e.power = Fx::from_int(t::reap_hurl_for() as i32);
                self.effects[slot] = Some(e);
            } else if e.is_a_pool() && e.age >= t::stain_settles() {
                drunk += e.banked;
                self.effects[slot] = None;
            }
        }
        if drunk > 0 {
            self.players[i].absorb(drunk);
        }
        for j in 0..MAX_PLAYERS {
            let q = self.players[j];
            if !self.effects_reach(j, i as u8) || q.action.invulnerable() || !in_cone(q.pos) {
                continue;
            }
            // Up the cone as far as it reaches out.
            if q.pos.y.raw() > p.pos.y.add(m.reach).raw() {
                continue;
            }
            let dealt = self.players[j].wound(m.damage);
            self.players[j].vel.y = self.players[j].vel.y.max(m.launch);
            self.players[j].grounded = false;
            self.spill(i as u8, p.class, moves::blood::RISING_REAP, q.pos, dealt);
        }
        let half = Fx::from_raw(m.reach.raw() / 2);
        let mid = p.pos.add(dir.scale(half));
        let mut sweep = Effect::cast(
            EffectKind::Cascade,
            i as u8,
            p.class,
            moves::blood::RISING_REAP,
            p.pos,
            dir,
            m.reach,
        );
        sweep.life = t::spell_flash();
        self.gore_each(&mut sweep, 0, mid, half, Fx::ONE);
        self.gore_critters(&mut sweep, mid, half, Fx::ONE);
        spawn_effect(&mut self.effects, sweep);
    }

    /// The blow out on a thrall this frame, if any: the most of any enemy
    /// fighter's attack volume touching it, or the creature's. Nought if
    /// none.
    fn blow_on_a_thrall(&self, th: &Effect, radius: Fx, height: Fx) -> i32 {
        let mut most = 0;
        for j in 0..MAX_PLAYERS {
            let q = &self.players[j];
            if j as u8 == th.owner
                || q.health <= 0
                || !self.pvp()
                || (th.owner as usize) < MAX_PLAYERS && !self.foes(j, th.owner as usize)
            {
                continue;
            }
            let Some(hb) = hitbox(q) else { continue };
            let reach = hb.radius.add(radius);
            let top = V3::new(th.pos.x, th.pos.y.add(height), th.pos.z);
            let touched = if hb.flat {
                th.pos.sub(hb.flat_nearest(th.pos)).flat_len().raw() <= reach.raw()
            } else {
                crate::math::segment_gap(hb.from, hb.to, th.pos, top).raw() <= reach.raw()
            };
            if touched {
                if let Action::Active { kind, .. } = q.action {
                    most = most.max(moves::get(q.class, kind).damage.max(1));
                }
            }
        }
        for beast in self.monsters.iter().flatten() {
            if beast.alive() && beast.reaches(th.pos, height, radius) {
                if let monster::Doing::Active { kind, .. } = beast.doing {
                    most = most.max(beast.sp().attack(kind).damage.max(1));
                }
            }
        }
        most
    }

    /// The floor under the crosshair, where a grounded move of hers lands.
    fn floor_at_aim(&self, i: usize) -> V3 {
        let at = self.players[i].aim_at();
        let lift = V3::new(at.x, at.y.add(Fx::ONE), at.z);
        V3::new(at.x, self.terrain().ground_under(lift), at.z)
    }

    /// **A finisher's blow**: the row's own damage, and
    /// `tuning::finisher_per_grey` for every point of grey she spent
    /// on it (`Player::fuel`). Always more than the row; more the more she had.
    fn finisher_blow(&self, i: usize, kind: u8) -> i32 {
        let p = self.players[i];
        let m = moves::get(p.class, kind);
        m.damage
            + Fx::from_int(p.fuel as i32)
                .mul(t::finisher_per_grey())
                .to_int()
    }

    /// **The Bramble is laid**: a field of thorns on the floor where she is
    /// aiming, growing while she holds (`step_plagues`). One of hers at a
    /// time.
    fn lay_the_plague(&mut self, i: usize) {
        let p = self.players[i];
        let floor = self.floor_at_aim(i);
        let mut cloud = Effect::cast(
            EffectKind::Plague,
            i as u8,
            p.class,
            moves::blood::BRAMBLE,
            floor,
            V3::ZERO,
            t::plague_from(),
        );
        cloud.banked = GROWING;
        spawn_the_only_one(&mut self.effects, cloud);
    }

    /// How far into her hold of `kind` she is, nought to one, while she is
    /// holding it; `None` once she is not.
    fn hold_of(&self, owner: usize, kind: u8) -> Option<Fx> {
        match self.players.get(owner).map(|p| p.action) {
            Some(Action::Channel { kind: k, held }) if k == kind => {
                let cap = moves::get(self.players[owner].class, kind).channel.max(1);
                Some(Fx::ratio(held.min(cap) as i32, cap as i32))
            }
            _ => None,
        }
    }

    /// Is she in the finisher `kind` -- winding it up or throwing it?
    fn finishing(&self, owner: usize, kind: u8) -> bool {
        self.players
            .get(owner)
            .and_then(|p| p.action.attack_kind())
            .is_some_and(|k| k == kind)
    }

    /// **Every growing Bramble, one frame.** While she holds it, it grows
    /// from `tuning::plague_from` to `plague_to` and its life does not run.
    /// Let go, it stands at the size it reached for `plague_lasts`. Held to
    /// its end, it becomes the Black vines.
    fn step_plagues(&mut self) {
        for slot in 0..MAX_EFFECTS {
            let Some(mut cloud) =
                self.effects[slot].filter(|e| e.kind == EffectKind::Plague && e.banked == GROWING)
            else {
                continue;
            };
            let owner = cloud.owner as usize;
            if let Some(share) = self.hold_of(owner, moves::blood::BRAMBLE) {
                cloud.reach = t::plague_from().add(t::plague_to().sub(t::plague_from()).mul(share));
                // Its life does not run while she holds it.
                cloud.life = cloud.life.saturating_add(1);
            } else if self.finishing(owner, moves::blood::VINES) {
                self.effects[slot] = None;
                self.grow_the_vines(owner, &cloud);
                continue;
            } else {
                cloud.banked = 0;
            }
            self.effects[slot] = Some(cloud);
        }
    }

    /// **The Black vines**: out of the cloud's middle to `tuning::vines_size`
    /// of its radius, carrying the finisher's blow.
    fn grow_the_vines(&mut self, owner: usize, cloud: &Effect) {
        let blow = self.finisher_blow(owner, moves::blood::VINES);
        self.players[owner].fuel = 0;
        let mut vines = Effect::cast(
            EffectKind::Vines,
            owner as u8,
            cloud.class,
            moves::blood::VINES,
            cloud.pos,
            V3::ZERO,
            cloud.reach.mul(t::vines_size()),
        );
        vines.banked = blow;
        spawn_the_only_one(&mut self.effects, vines);
    }

    /// How far out the vines have got: the whole reach after
    /// `tuning::vines_grow` frames.
    fn vines_front(vines: &Effect) -> Fx {
        let grow = t::vines_grow().max(1);
        vines
            .reach
            .mul(Fx::ratio(vines.age.min(grow) as i32, grow as i32))
    }

    /// **The Black vines, one frame.** As the front passes a body it takes
    /// the blow and a bleed, once; while they stand, everything in them is
    /// bled and slowed every `tuning::vines_tick`.
    fn step_the_vines(&mut self, vines: &mut Effect) {
        let front = Self::vines_front(vines);
        let growing = vines.age <= t::vines_grow();
        let tick = vines.age % t::vines_tick().max(1) == 0;
        // The floor the vines stand in is hers.
        if tick {
            self.lay_touch(vines.owner as usize, vines.pos, front);
        }
        for j in 0..MAX_PLAYERS {
            let q = self.players[j];
            if !self.effects_reach(j, vines.owner) || q.action.invulnerable() {
                continue;
            }
            let flat = V3::new(q.pos.x.sub(vines.pos.x), Fx::ZERO, q.pos.z.sub(vines.pos.z));
            if flat.flat_len().raw() > front.add(t::body_radius()).raw()
                || q.pos.y.raw() > vines.pos.y.add(t::body_height()).raw()
            {
                continue;
            }
            if !vines.already_hit(0, j) {
                vines.take_hit(0, j);
                let dealt = self.players[j].wound(vines.banked);
                self.spill(vines.owner, vines.class, vines.slot, q.pos, dealt);
                self.players[vines.owner as usize].heal(vines.leeched(dealt));
                self.players[j].bleed(vines.owner, t::bleed_lasts());
            }
            if tick {
                self.players[j].bleed(vines.owner, t::bleed_lasts());
                self.players[j].slow(t::vines_tick().saturating_add(10), t::plague_slow());
            }
        }
        // The creature: the blow once, as the front reaches the middle of
        // the vines' reach, and the bleed and the slow on every tick after.
        let m = vines.source();
        if growing && vines.age == t::vines_grow().max(2) / 2 {
            let share = Fx::ratio(vines.banked, m.damage.max(1));
            let each = self.gore_each(vines, 0, vines.pos, vines.reach, share);
            vines.forget_hits();
            let dealt =
                each.iter().sum::<i32>() + self.gore_critters(vines, vines.pos, vines.reach, share);
            self.players[vines.owner as usize].heal(vines.leeched(dealt));
        }
        if tick {
            for slot in 0..crate::monster::MAX_MONSTERS {
                let Some(mut beast) = self.monsters[slot] else {
                    continue;
                };
                if !beast.alive()
                    || beast
                        .part_struck(vines.pos, front, t::body_height())
                        .is_none()
                {
                    continue;
                }
                beast.bleeding = beast.bleeding.max(t::bleed_lasts());
                beast.bled_by = vines.owner;
                beast.take_control(vines.control());
                self.monsters[slot] = Some(beast);
            }
        }
    }

    /// Her stains with their middles within `reach` of `at`, nearest first,
    /// as slots.
    fn stains_within(
        &self,
        owner: usize,
        at: V3,
        reach: Fx,
    ) -> impl Iterator<Item = usize> + use<> {
        let mut found = [(usize::MAX, i32::MAX); MAX_EFFECTS];
        for (slot, e) in self.effects.iter().enumerate() {
            let Some(e) = e else { continue };
            if !e.is_a_pool() || e.owner as usize != owner {
                continue;
            }
            let far = V3::new(e.pos.x.sub(at.x), Fx::ZERO, e.pos.z.sub(at.z)).flat_len();
            if far.raw() <= reach.raw() {
                found[slot] = (slot, far.raw());
            }
        }
        found.sort_unstable_by_key(|(_, d)| *d);
        found
            .into_iter()
            .filter(|(s, _)| *s != usize::MAX)
            .map(|(s, _)| s)
    }

    /// How many thralls she has standing.
    fn thralls_of(&self, owner: usize) -> i32 {
        self.effects
            .iter()
            .flatten()
            .filter(|e| e.kind == EffectKind::Thrall && e.owner as usize == owner)
            .count() as i32
    }

    /// The stain in `slot` stands up: a thrall of exactly its essence, in its
    /// own slot.
    fn stand_up(&mut self, slot: usize) {
        if let Some(e) = self.effects[slot].as_mut() {
            e.kind = EffectKind::Thrall;
            e.slot = moves::blood::HEART;
            e.home = e.pos;
            e.age = 0;
            e.life = EffectKind::Thrall.life();
            e.power = Fx::ZERO;
            e.struck.clear();
            e.dir = V3::ZERO;
        }
    }

    /// **The Crimson cascade** (2026-10-14): the impact at the floor under
    /// the crosshair erupts for the finisher's blow, and spills
    /// `tuning::cascade_first` pools round its edge holding all the grey
    /// she spent between them (`Player::fuel`). Each erupts
    /// `cascade_fuse` frames later and spills `cascade_branch` smaller pools
    /// further out (`erupt_from`), for `cascade_generations`. Every pool it
    /// spills **reaches for her nearest stain** within `cascade_track`,
    /// however small, so a bleed's trail is followed to its end.
    fn start_the_cascade(&mut self, i: usize) {
        let p = self.players[i];
        let to = self.floor_at_aim(i);
        let m = moves::get(p.class, moves::blood::CASCADE);
        let blow = self.finisher_blow(i, moves::blood::CASCADE);
        let fuel = (p.fuel as i32).max(0);
        self.players[i].fuel = 0;
        let mut impact = Effect::cast(
            EffectKind::BlackSpike,
            i as u8,
            p.class,
            moves::blood::BLACK_SPIKE,
            to,
            V3::ZERO,
            t::cascade_step(),
        );
        let row = moves::get(p.class, moves::blood::BLACK_SPIKE).damage.max(1);
        impact.banked = SPIKE_SCALED + blow * PERCENT / row;
        spawn_effect(&mut self.effects, impact);
        self.light_round(i, to, t::cascade_step());
        let n = t::cascade_first().max(1);
        let each = (fuel / n).max(t::cascade_least()).max(m.damage / n);
        let ahead = V3::new(p.facing.x, Fx::ZERO, p.facing.z).normalized();
        self.spill_the_cascade(i, to, ahead, n, each, 1, Fx::ONE);
    }

    /// Light every unlit stain of hers within `reach` of `at` (and its own
    /// edge) to go up `tuning::chain_delay` frames from now.
    fn light_round(&mut self, owner: usize, at: V3, reach: Fx) {
        for e in self.effects.iter_mut().flatten() {
            if e.is_a_pool()
                && e.owner == owner as u8
                && e.life == UNLIT
                && V3::new(e.pos.x.sub(at.x), Fx::ZERO, e.pos.z.sub(at.z))
                    .flat_len()
                    .raw()
                    <= reach.add(e.pool_radius()).add(t::erupt_chain()).raw()
            {
                e.life = t::chain_delay().max(1);
            }
        }
    }

    /// **A cascade spills `n` pools of `each` from `from`**, generation
    /// `generation`. Each goes **toward her nearest unlit stain** within
    /// `tuning::cascade_track` that no other of these is going for -- a step
    /// of `cascade_step` along the way, or lighting it outright if it is
    /// that close -- and the rest fan out round `ahead` across `spread` of
    /// a turn, a little off even, so it grows like roots rather than rings.
    #[allow(clippy::too_many_arguments)]
    fn spill_the_cascade(
        &mut self,
        owner: usize,
        from: V3,
        ahead: V3,
        n: i32,
        each: i32,
        generation: i32,
        spread: Fx,
    ) {
        let step = t::cascade_step();
        let mut taken = [false; MAX_EFFECTS];
        for k in 0..n {
            // Her nearest stain in reach not already being gone for.
            let mut target: Option<(usize, V3, Fx)> = None;
            for (slot, e) in self.effects.iter().enumerate() {
                let Some(e) = e else { continue };
                if !e.is_a_pool()
                    || e.owner != owner as u8
                    || e.life != UNLIT
                    || e.slot == moves::blood::CASCADE
                    || taken[slot]
                {
                    continue;
                }
                let far = V3::new(e.pos.x.sub(from.x), Fx::ZERO, e.pos.z.sub(from.z)).flat_len();
                if far.raw() <= t::cascade_track().raw()
                    && target.is_none_or(|(_, _, b)| far.raw() < b.raw())
                {
                    target = Some((slot, e.pos, far));
                }
            }
            let way = match target {
                Some((slot, at, far)) => {
                    taken[slot] = true;
                    if far.raw() <= step.add(t::body_radius()).raw() {
                        if let Some(e) = self.effects[slot].as_mut() {
                            e.life = t::cascade_fuse().max(1);
                        }
                        continue;
                    }
                    V3::new(at.x.sub(from.x), Fx::ZERO, at.z.sub(from.z)).normalized()
                }
                None => {
                    // Fanned round `ahead`, and a little crooked: the turn is
                    // nudged by a few hundredths that depend on where it is.
                    let n_f = n.max(1);
                    let base = if spread.raw() >= Fx::ONE.raw() {
                        Fx::ratio(k, n_f)
                    } else {
                        spread.mul(Fx::ratio(2 * k + 1 - n_f, 2 * n_f))
                    };
                    let wobble = ((from.x.raw() ^ from.z.raw().rotate_left(7)) as u32 >> 8) % 9;
                    let turn = base.add(Fx::ratio(wobble as i32 - 4, 100));
                    let (c, s) = (crate::fixed::cos_turns(turn), crate::fixed::sin_turns(turn));
                    V3::new(
                        ahead.x.mul(c).sub(ahead.z.mul(s)),
                        Fx::ZERO,
                        ahead.x.mul(s).add(ahead.z.mul(c)),
                    )
                }
            };
            let flat = from.add(way.scale(step));
            let lift = V3::new(flat.x, from.y.add(Fx::ONE), flat.z);
            let at = V3::new(flat.x, self.terrain().ground_under(lift), flat.z);
            let class = self.players[owner].class;
            let mut pool = Effect::pool(owner as u8, class, moves::blood::CASCADE, at, each);
            pool.life = t::cascade_fuse().max(1);
            pool.power = Fx::from_int(generation);
            pool.dir = way;
            pool.age = t::stain_settles().saturating_add(1);
            spawn_effect(&mut self.effects, pool);
        }
    }

    /// A spike came up on the pool in `slot`: the pool erupts, at its own
    /// size, and every pool of hers the eruption covers is lit to erupt
    /// `tuning::chain_delay` frames later (`light_the_fuses`).
    ///
    /// **The chain is the skill curve.** A spike on one pool is a bigger spike;
    /// a spike on a pool standing among others is the floor coming up across
    /// the whole fight, running outward link by link. Each eruption drinks its
    /// own pool, so the chain cannot run twice over the same blood.
    ///
    /// **A Crimson cascade's pool** is her spent grey, not blood she won: it
    /// is not drunk, it hits for what it held (`tuning::cascade_power`), and
    /// it spills the next generation (`spill_the_cascade`).
    fn erupt_from(&mut self, owner: usize, kind: u8, slot: usize) {
        let Some(pool) = self.effects[slot].filter(|e| e.is_a_pool()) else {
            return;
        };
        let class = self.players[owner].class;
        let m = moves::get(class, kind);
        let radius = t::erupt_radius()
            .mul(Fx::from_int(pool.pool_volume().max(0)).sqrt())
            .max(m.radius);
        let cascade = pool.slot == moves::blood::CASCADE;
        if !cascade {
            self.drink_from(slot, owner, &m);
        }
        self.effects[slot] = None;
        let mut born = Effect::cast(
            EffectKind::BlackSpike,
            owner as u8,
            class,
            kind,
            pool.pos,
            V3::ZERO,
            radius,
        );
        born.banked = if cascade {
            let share =
                Fx::ratio(pool.banked.max(0), t::pool_full().max(1)).mul(t::cascade_power());
            SPIKE_SCALED + share.mul(Fx::from_int(PERCENT)).to_int()
        } else {
            SPIKE_ERUPTION
        };
        // Her ground's strength as she stands now: a chain she set off is
        // her blow (2026-10-19). What it sets off, it sets off as it spreads
        // (`Effect::spreads`).
        born.power = worth_of(&self.players[owner], None);
        // **Worth its stain** (2026-10-19): the blow is the blood that was in
        // it, `tuning::stain_erupts` to the point, rather than the row's --
        // carried on the power, which every hit an effect lands is scaled by.
        if !cascade {
            let row = Fx::from_int(m.damage.max(1)).mul(t::erupt_damage());
            let worth = Fx::from_int(pool.pool_volume().max(0)).mul(t::stain_erupts());
            born.power = born.power.mul(worth.div(row.max(Fx::ONE)));
        }
        spawn_effect(&mut self.effects, born);
        if cascade {
            let generation = pool.power.to_int();
            let next = Fx::from_int(pool.banked).mul(t::cascade_share()).to_int();
            if generation < t::cascade_generations() && next >= t::cascade_least() {
                let ahead = if pool.dir == V3::ZERO {
                    V3::new(Fx::ONE, Fx::ZERO, Fx::ZERO)
                } else {
                    pool.dir
                };
                self.spill_the_cascade(
                    owner,
                    pool.pos,
                    ahead,
                    t::cascade_branch(),
                    next,
                    generation + 1,
                    Fx::ratio(1, 4),
                );
            }
        }
    }

    /// **A Black spike's blow erupts** (2026-10-19): a spike of the row
    /// (`moves::blood::BLACK_SPIKE`, tapped or one of a held swarm or the
    /// first wave of its field) that dealt `dealt` comes up again at once
    /// as an eruption sized by that, worth `tuning::spike_hit_erupts` of it.
    /// It needs no stain under it -- it is the stain it would have dropped,
    /// set off -- and it spreads, setting off every stain of hers its edge
    /// reaches: the bleed laid them, and this is what finishes them.
    fn erupt_from_a_hit(&mut self, spike: &Effect, dealt: i32) {
        if spike.class != Class::BloodMage
            || spike.slot != moves::blood::BLACK_SPIKE
            || !matches!(spike.banked, SPIKE_AREA | SPIKE_FIELD)
            || (spike.owner as usize) >= MAX_PLAYERS
        {
            return;
        }
        let m = moves::get(spike.class, moves::blood::BLACK_SPIKE);
        let worth = Fx::from_int(dealt).mul(t::spike_hit_erupts()).to_int();
        if worth <= 0 {
            return;
        }
        let radius = t::erupt_radius()
            .mul(Fx::from_int(dealt).sqrt())
            .max(m.radius);
        let mut born = Effect::cast(
            EffectKind::BlackSpike,
            spike.owner,
            spike.class,
            moves::blood::BLACK_SPIKE,
            spike.pos,
            V3::ZERO,
            radius,
        );
        // Out of what was dealt, which already carried her strength.
        born.banked = SPIKE_ECHO + worth * PERCENT / m.damage.max(1);
        spawn_effect(&mut self.effects, born);
    }

    /// **Lit stains erupt** when their fuse runs out: a stain set off by an
    /// eruption beside it waits `tuning::chain_delay` frames first, so a
    /// chain is seen to run from one place to the next.
    fn light_the_fuses(&mut self) {
        for slot in 0..MAX_EFFECTS {
            let Some(e) = self.effects[slot].filter(|e| e.is_a_pool() && e.life != UNLIT) else {
                continue;
            };
            if e.life > 1 {
                if let Some(pool) = self.effects[slot].as_mut() {
                    pool.life -= 1;
                }
                continue;
            }
            let owner = e.owner as usize;
            if owner < MAX_PLAYERS {
                self.erupt_from(owner, moves::blood::BLACK_SPIKE, slot);
            } else {
                self.effects[slot] = None;
            }
        }
    }

    /// **The Black spike, let go** (2026-10-13): one spike for a tap, and
    /// more the longer the hold -- up to `tuning::spikes_most` -- the first at
    /// the aimed point and the rest on a ring round it, as wide as
    /// `spike_spread_most` at a full hold. A spike on a stain erupts it.
    fn raise_the_tier(&mut self, i: usize, born: Effect) {
        let p = self.players[i];
        let share = p.channelled.max(Fx::ZERO).min(Fx::ONE);
        let more = Fx::from_int((t::spikes_most() - 1).max(0))
            .mul(share)
            .add(Fx::ratio(1, 2))
            .to_int()
            .max(0);
        let ring = t::spike_spread_most().mul(share);
        let f = p.facing;
        let ahead = V3::new(f.x, Fx::ZERO, f.z).normalized();
        let side = V3::new(Fx::ZERO.sub(ahead.z), Fx::ZERO, ahead.x);
        let centre = born.pos;
        // **The swarm goes for her stains first** (2026-10-14): the spikes
        // past the first land on stains of hers inside the ring, nearest
        // first, however small; only what is left stands on the ring.
        let mut on_stains = 0;
        if more > 0 {
            let reach = ring.add(t::body_radius());
            for slot in self.stains_within(i, centre, reach) {
                if on_stains >= more {
                    break;
                }
                if self.effects[slot].is_some_and(|e| e.is_a_pool() && e.life == UNLIT) {
                    self.erupt_from(i, moves::blood::BLACK_SPIKE, slot);
                    on_stains += 1;
                }
            }
        }
        for k in 0..=(more - on_stains) {
            let at = if k == 0 {
                centre
            } else {
                let turn = Fx::ratio(k - 1, (more - on_stains).max(1));
                let (c, s) = (crate::fixed::cos_turns(turn), crate::fixed::sin_turns(turn));
                let off = ahead.scale(c).add(side.scale(s)).scale(ring);
                let flat = V3::new(
                    centre.x.add(off.x),
                    centre.y.add(Fx::ONE),
                    centre.z.add(off.z),
                );
                V3::new(flat.x, self.terrain().ground_under(flat), flat.z)
            };
            let reach = moves::get(p.class, moves::blood::BLACK_SPIKE).radius;
            if let Some(slot) = self.pool_touching(i as u8, at, reach) {
                self.erupt_from(i, moves::blood::BLACK_SPIKE, slot);
                continue;
            }
            let mut spike = born;
            spike.pos = at;
            spike.reach = moves::get(p.class, moves::blood::BLACK_SPIKE).radius;
            spike.banked = SPIKE_AREA;
            spawn_effect(&mut self.effects, spike);
        }
    }

    /// **Every thrall, one frame** (2026-10-13, 2026-10-18).
    ///
    /// **On her ground it is stronger** (`ON_GROUND`): it runs at
    /// `tuning::thrall_ground_speed`, bites at `thrall_ground_bite`, and does
    /// not drain. Hurled by a Rising reap, it flies. Otherwise it goes for
    /// **the nearest enemy body** within `tuning::thrall_seek` that is on her
    /// ground or within `thrall_leash` of it -- a fighter, the creature or
    /// one of a pack -- and in reach it bites every `thrall_bite_every`: **a
    /// bite spends what it deals**, and spills exactly that under its prey.
    /// With nothing to bite, a thrall off her ground walks back to it. It
    /// walks round walls (`walk_round`). Any blow out on a thrall kills it,
    /// back into a stain of all it had.
    fn step_thralls(&mut self) {
        for slot in 0..MAX_EFFECTS {
            let Some(mut th) = self.effects[slot].filter(|e| e.is_a_thrall()) else {
                continue;
            };
            let owner = th.owner as usize;
            let (radius, height) = (th.thrall_radius(), th.thrall_height());
            // Struck?
            let blow = self.blow_on_a_thrall(&th, radius, height);
            // **Killed, it leaves a stain of all it had** (2026-10-19), not
            // what the blow left over: what a thrall has is what it was not
            // drained of, so one that reached them over her ground leaves a
            // bigger stain -- new ground, and something for a spike.
            if blow > 0 {
                self.effects[slot] = None;
                if th.banked > 0 {
                    self.pool_blood(th.owner, th.class, th.slot, th.pos, th.banked);
                }
                continue;
            }
            let home = owner < MAX_PLAYERS && self.on_blood(owner, th.pos);
            if home {
                th.struck.set(ON_GROUND);
            } else {
                th.struck.unset(ON_GROUND);
            }
            let flying = th.power.raw() > 0;
            let prey = self.prey_of(&th);
            let speed = if home {
                t::thrall_speed().mul(t::thrall_ground_speed())
            } else {
                t::thrall_speed()
            };
            if flying {
                th.power = th.power.sub(Fx::ONE);
                let to = th.pos.add(th.dir.scale(t::reap_hurl().mul(DT)));
                th.pos = self.walk_to(th.pos, to, radius, height);
            } else {
                // Its prey; or, with none and off her ground, back to it.
                let back = if prey.is_none() && !home && owner < MAX_PLAYERS {
                    self.nearest_blood(owner, th.pos)
                } else {
                    None
                };
                let goal = match (prey, back) {
                    (Some((_, at)), _) => Some((at, t::thrall_reach().mul(Fx::ratio(1, 2)))),
                    (None, Some(at)) => Some((at, Fx::ZERO)),
                    (None, None) => None,
                };
                if let Some((goal, near)) = goal {
                    let gap = V3::new(goal.x.sub(th.pos.x), Fx::ZERO, goal.z.sub(th.pos.z));
                    let far = gap.flat_len();
                    if far.raw() > near.add(radius).raw() {
                        let dir = gap.normalized();
                        let step = speed.mul(DT).min(far.sub(near).max(Fx::ZERO));
                        let (to, way, left) = self.walk_round(
                            th.pos,
                            dir,
                            step,
                            radius,
                            height,
                            th.struck.has(LEFT_HAND),
                        );
                        th.pos = to;
                        if way != V3::ZERO {
                            th.dir = way;
                        }
                        if left {
                            th.struck.set(LEFT_HAND);
                        } else {
                            th.struck.unset(LEFT_HAND);
                        }
                    } else if prey.is_some() {
                        th.dir = gap.normalized();
                    }
                }
                th.home = th.pos;
            }
            // Bite.
            if let Some((who, at)) = prey {
                let gap = V3::new(at.x.sub(th.pos.x), Fx::ZERO, at.z.sub(th.pos.z)).flat_len();
                if gap.raw() <= t::thrall_reach().add(radius).raw()
                    && (th.age + 1) % t::thrall_bite_every().max(1) == 0
                {
                    self.bite(&mut th, who, at, owner);
                }
            }
            if th.banked <= 0 {
                self.effects[slot] = None;
            } else {
                self.effects[slot] = Some(th);
            }
        }
    }

    /// One step of a thrall's walk from `from` to `to`, pushed out of what it
    /// would walk into. Where it ends up.
    fn walk_to(&self, from: V3, to: V3, radius: Fx, height: Fx) -> V3 {
        let lift = V3::new(to.x, from.y.add(Fx::ratio(1, 2)), to.z);
        let moved = self
            .terrain()
            .resolve_sized(lift, V3::ZERO, true, radius, height)
            .pos;
        V3::new(moved.x, self.terrain().ground_under(moved), moved.z)
    }

    /// **A thrall's way round a wall**: straight at what it wants if it can;
    /// if a step that way is blocked -- pushed back, or up more than it can
    /// climb -- the nearest heading that is not, turning first to the hand it
    /// turned to last (so it follows a wall rather than dithering at it), out
    /// to a third of a turn either side. Where it ends up, which way it went,
    /// and which hand it is keeping to.
    fn walk_round(
        &self,
        from: V3,
        dir: V3,
        step: Fx,
        radius: Fx,
        height: Fx,
        left: bool,
    ) -> (V3, V3, bool) {
        let climb = t::terrain_steepest()
            .mul(step)
            .max(crate::arena::SKIN)
            .add(Fx::ratio(1, 4));
        let try_way = |way: V3| {
            let to = self.walk_to(from, from.add(way.scale(step)), radius, height);
            let went = V3::new(to.x.sub(from.x), Fx::ZERO, to.z.sub(from.z)).flat_len();
            (went.raw() * 10 >= step.raw() * 6 && to.y.sub(from.y).raw() <= climb.raw())
                .then_some(to)
        };
        if let Some(to) = try_way(dir) {
            return (to, dir, left);
        }
        let turns = [
            Fx::ratio(1, 12),
            Fx::ratio(1, 6),
            Fx::ratio(1, 4),
            Fx::ratio(1, 3),
        ];
        for hand in [left, !left] {
            for turn in turns {
                let a = if hand { turn } else { Fx::ZERO.sub(turn) };
                let (c, s) = (crate::fixed::cos_turns(a), crate::fixed::sin_turns(a));
                let way = V3::new(
                    dir.x.mul(c).sub(dir.z.mul(s)),
                    Fx::ZERO,
                    dir.x.mul(s).add(dir.z.mul(c)),
                );
                if let Some(to) = try_way(way) {
                    return (to, way, hand);
                }
            }
        }
        (from, dir, !left)
    }

    /// **One bite.** A thrall spends `tuning::thrall_bite`% of its essence
    /// (never less than `thrall_bite_least`), and on her ground the blow is
    /// `thrall_ground_bite` times that; the stain it spills is the blow.
    /// `who` is a fighter's index, `MAX_PLAYERS + slot` for a creature, and
    /// past those a pack's critter.
    fn bite(&mut self, th: &mut Effect, who: usize, at: V3, owner: usize) {
        let x = (th.banked * t::thrall_bite() / 100)
            .max(t::thrall_bite_least())
            .min(th.banked);
        if x <= 0 {
            return;
        }
        let blow = if th.struck.has(ON_GROUND) {
            Fx::from_int(x).mul(t::thrall_ground_bite()).to_int()
        } else {
            x
        };
        let dealt = self.bite_for(th, who, at, owner, blow);
        if dealt > 0 {
            th.banked -= x;
        }
    }

    /// The blow of a bite, `x`, on `who`: what it dealt.
    fn bite_for(&mut self, th: &Effect, who: usize, at: V3, owner: usize, x: i32) -> i32 {
        if who < MAX_PLAYERS {
            if self.players[who].action.invulnerable() {
                return 0;
            }
            let dealt = self.players[who].wound(x);
            let floor = self.players[who].pos;
            self.spill(owner as u8, th.class, th.slot, floor, dealt);
            return dealt;
        }
        if who < MAX_PLAYERS + crate::monster::MAX_MONSTERS {
            let slot = who - MAX_PLAYERS;
            let Some(mut beast) = self.monsters[slot] else {
                return 0;
            };
            if !beast.alive() {
                return 0;
            }
            let Some(struck) = beast.part_struck(
                th.pos,
                t::thrall_reach().add(th.thrall_radius()),
                th.thrall_height().max(t::body_height()),
            ) else {
                return 0;
            };
            let (dealt, guarded) = beast.take_blow(
                struck,
                x,
                &monster::Blow {
                    from: th.pos,
                    unblockable: false,
                    who: th.owner,
                    class: th.class,
                    kind: th.slot,
                },
            );
            self.monsters[slot] = Some(beast);
            if dealt > 0 || !guarded.turned() {
                self.spill(
                    th.owner,
                    th.class,
                    th.slot,
                    floor_under(self.arena(), at),
                    dealt.max(x),
                );
            }
            return dealt.max(x);
        }
        let c = who - MAX_PLAYERS - crate::monster::MAX_MONSTERS;
        if !self.critters[c].alive() {
            return 0;
        }
        let body = self.critters[c];
        let away =
            V3::new(body.pos.x.sub(th.pos.x), Fx::ZERO, body.pos.z.sub(th.pos.z)).normalized();
        let dealt = pack::hurt(
            &mut self.pack,
            &mut self.critters,
            c,
            x,
            away,
            Fx::ZERO,
            Fx::ZERO,
        );
        if dealt > 0 {
            self.spill(
                th.owner,
                th.class,
                th.slot,
                floor_under(self.arena(), body.pos),
                dealt.max(x),
            );
        }
        dealt
    }

    /// The body a thrall goes for: **the nearest enemy** within
    /// `tuning::thrall_seek` of it -- a fighter open to its owner, the
    /// creature's nearest part, or a pack's critter. With where it is.
    fn prey_of(&self, th: &Effect) -> Option<(usize, V3)> {
        let seek = t::thrall_seek();
        let owner = th.owner as usize;
        // **It keeps to her ground** (2026-10-18): only what is on it, or
        // within `tuning::thrall_leash` of it.
        let near_home =
            |at: V3| owner < MAX_PLAYERS && self.near_blood(owner, at, t::thrall_leash());
        let mut best: Option<(usize, V3, Fx)> = None;
        let mut offer = |who: usize, at: V3| {
            let far = V3::new(at.x.sub(th.pos.x), Fx::ZERO, at.z.sub(th.pos.z)).flat_len();
            if far.raw() <= seek.raw()
                && best.is_none_or(|(_, _, b)| far.raw() < b.raw())
                && near_home(at)
            {
                best = Some((who, at, far));
            }
        };
        for j in 0..MAX_PLAYERS {
            if self.effects_reach(j, th.owner) {
                offer(j, self.players[j].pos);
            }
        }
        for (slot, beast) in self.monsters.iter().enumerate() {
            if let Some(beast) = beast.filter(|b| b.alive()) {
                offer(MAX_PLAYERS + slot, beast.nearest_to(th.pos));
            }
        }
        if self.critters.any() {
            for c in 0..critter::MAX_CRITTERS {
                let body = self.critters[c];
                if body.alive() {
                    offer(MAX_PLAYERS + crate::monster::MAX_MONSTERS + c, body.pos);
                }
            }
        }
        best.map(|(who, at, _)| (who, at))
    }

    /// **The Stake** leaves her hand along the solved line, to the point the
    /// crosshair was on: through anybody in the air on the way, pinning them,
    /// and into the floor at the end, where it lands in an area
    /// (`pay_out`).
    fn throw_the_stake(&mut self, i: usize) {
        let p = self.players[i];
        let path = p.aim_path;
        spawn_effect(
            &mut self.effects,
            Effect::cast(
                EffectKind::Nail,
                i as u8,
                p.class,
                moves::blood::STAKE,
                path.from,
                path.dir(),
                path.length().max(Fx::ONE),
            ),
        );
    }

    /// **The Stake lands**: at the end of its flight, in the floor under the
    /// point the crosshair was on -- an area that hurts and slows, as a spike
    /// does. On a stain of hers, the stain erupts instead.
    fn land_the_stake(&mut self, effect: &Effect) {
        let owner = effect.owner as usize;
        if owner >= MAX_PLAYERS {
            return;
        }
        let end = effect.pos.add(effect.dir.scale(effect.reach));
        let lift = V3::new(end.x, end.y.add(Fx::ONE), end.z);
        let at = V3::new(end.x, self.terrain().ground_under(lift), end.z);
        if let Some(slot) = self.pool_covering(effect.owner, at) {
            self.erupt_from(owner, moves::blood::STAKE, slot);
            return;
        }
        let m = effect.source();
        let mut area = Effect::cast(
            EffectKind::BlackSpike,
            effect.owner,
            effect.class,
            moves::blood::STAKE,
            at,
            V3::ZERO,
            m.radius,
        );
        area.banked = SPIKE_AREA;
        spawn_effect(&mut self.effects, area);
    }

    /// **The Bramble's tick**: everyone inside the field is hurt by the
    /// row's number, and it spills into one stain at the centre; the creature
    /// is gored where the field reaches it; and **the floor under it is hers**
    /// for as long as a touch lasts from this tick.
    fn step_the_plague(&mut self, effect: &mut Effect) {
        let m = effect.source();
        let owner = effect.owner;
        self.lay_touch(owner as usize, effect.pos, effect.reach);
        let top = effect.pos.y.add(t::plague_height());
        for i in 0..MAX_PLAYERS {
            let p = self.players[i];
            if !self.effects_reach(i, owner) || p.action.invulnerable() {
                continue;
            }
            let flat = V3::new(
                p.pos.x.sub(effect.pos.x),
                Fx::ZERO,
                p.pos.z.sub(effect.pos.z),
            );
            if flat.flat_len().raw() > effect.reach.add(t::body_radius()).raw()
                || p.pos.y.raw() > top.raw()
                || p.pos.y.add(p.hurt_height()).raw() < effect.pos.y.raw()
            {
                continue;
            }
            let dealt = self.players[i].wound(m.damage);
            self.players[i].slow(t::plague_tick().saturating_add(10), t::plague_slow());
            self.spill(owner, effect.class, effect.slot, effect.pos, dealt);
        }
        effect.forget_hits();
        self.gore_each(effect, 0, effect.pos, effect.reach, Fx::ONE);
        effect.forget_hits();
        self.gore_critters(effect, effect.pos, effect.reach, Fx::ONE);
    }

    /// **Rebound**: the Bash in the air, and if the shield's face met
    /// anything solid within its reach -- a body, a wall, a stone, his own
    /// planted shield, the floor -- he is thrown back off it and up. Asked of
    /// the line the swing is thrown along.
    fn rebound(&mut self, i: usize) {
        let p = self.players[i];
        let m = moves::get(p.class, moves::bulwark::REBOUND);
        let path = aim::Path {
            from: p.aim_path.from,
            to: p.aim_path.from.add(p.aim_path.dir().scale(m.reach)),
        };
        if self.first_on_the_line(i, path, m.radius, true).raw() >= m.reach.raw() {
            return;
        }
        bounce(&mut self.players[i], path.dir());
    }

    /// **Shield step**: the shield is planted where he stands, at the size its
    /// weight makes it, and he springs off its top -- higher the heavier it
    /// is -- and on forward. It is a planted shield like any other: `E` calls
    /// it home.
    fn shield_step(&mut self, i: usize) {
        let p = self.players[i];
        let Some(Shield::Held { weight }) = p.shield() else {
            return;
        };
        let ahead = V3::new(p.facing.x, Fx::ZERO, p.facing.z).normalized();
        let floor = self.terrain().ground_under(p.pos);
        let base = V3::new(p.pos.x, floor, p.pos.z);
        let top = bulwark::wall(base, weight).top();
        let full = bulwark::fullness_of(weight);
        let lift = t::step_lift().add(t::step_lift_full().mul(full));
        let forward = ahead.scale(t::step_forward());
        let q = &mut self.players[i];
        q.mechanic = Mechanic::Shield(Shield::Planted { pos: base, weight });
        q.pos.y = top;
        q.vel = V3::new(forward.x, lift, forward.z);
        q.grounded = false;
        q.air_stall = 0;
    }

    /// **Phase**: she is through -- along the crosshair for the move's reach,
    /// stopped a body short of the first wall, platform or stone
    /// (`aim::phase_to`) -- and a flash of light is left where she was. It
    /// spends the airdodge, unless it is the Equinox's blink (`airdodge`
    /// false).
    ///
    /// **She comes out of it moving** (2026-10-12): `phase, keeps this speed`
    /// along the blink, flat, and rising at `phase, rises` at the least. The
    /// blink is where the jump attack goes, and what it leaves her with is how
    /// far the jump goes after it.
    fn phase(&mut self, i: usize, airdodge: bool) {
        let p = self.players[i];
        let m = moves::get(p.class, moves::dual::PHASE);
        let stones = stones::gather(&self.players);
        let seen = self.players;
        let effects = self.effects;
        let scene = aim::Scene {
            stones: &stones,
            players: &seen,
            effects: &effects,
            quarry: &self.monsters,
            critters: &self.critters,
            arena: &self.terrain(),
        };
        let to = aim::phase_to(i, p.aim_path.dir(), m.reach, &scene);
        let mid = V3::new(
            p.pos.x,
            p.pos.y.add(t::body_height().mul(Fx::ratio(1, 2))),
            p.pos.z,
        );
        let mut flash = Effect::cast(
            EffectKind::Burst,
            i as u8,
            p.class,
            moves::dual::PHASE,
            mid,
            V3::ZERO,
            m.radius.mul(p.thrown_at),
        );
        flash.power = p.thrown_at;
        spawn_effect(&mut self.effects, flash);
        let along = {
            let d = p.aim_path.dir();
            V3::new(d.x, Fx::ZERO, d.z).normalized()
        };
        let q = &mut self.players[i];
        q.pos = to;
        let carry = along.scale(t::phase_carry());
        q.vel = V3::new(carry.x, q.vel.y.max(Fx::ZERO).max(t::phase_rise()), carry.z);
        q.grounded = false;
        if airdodge {
            q.air_dodged = true;
        }
    }

    /// **Chain lightning** (2026-10-12): black lightning out of a cloud over
    /// her head, onto the body nearest the crosshair's point and from it to
    /// the nearest body it has not struck yet, and on, `chain lightning,
    /// bodies it strikes` at most -- each one a burst where it lands, which
    /// hurts and curses (`EffectKind::Burst`, slot [`moves::dual::CHAIN_LIGHTNING`]),
    /// with `home` the point its bolt came from so the picture runs from link
    /// to link. Where it goes is `aim::chain_next`'s. With nobody there it
    /// strikes the crosshair's point.
    fn chain_lightning(&mut self, i: usize) {
        let p = self.players[i];
        let versus = self.pvp();
        let mut links = [None::<(V3, V3)>; 6];
        {
            let stones = stones::gather(&self.players);
            let seen = self.players;
            let effects = self.effects;
            let scene = aim::Scene {
                stones: &stones,
                players: &seen,
                effects: &effects,
                quarry: &self.monsters,
                critters: &self.critters,
                arena: &self.terrain(),
            };
            let cloud = aim::overhead(p.pos);
            let (mut from, mut near, mut reach) = (cloud, p.aim_at(), t::chain_first());
            let mut spent = 0u32;
            for link in links.iter_mut().take(t::chain_links()) {
                let Some((bit, at)) = aim::chain_next(from, near, reach, i, spent, versus, &scene)
                else {
                    break;
                };
                spent |= 1 << bit;
                *link = Some((from, at));
                (from, near, reach) = (at, at, t::chain_hop());
            }
            if links[0].is_none() {
                links[0] = Some((cloud, p.aim_at()));
            }
        }
        for (from, at) in links.into_iter().flatten() {
            let mut bolt = Effect::cast(
                EffectKind::Burst,
                i as u8,
                p.class,
                moves::dual::CHAIN_LIGHTNING,
                at,
                V3::ZERO,
                t::chain_radius(),
            );
            bolt.home = from;
            bolt.power = p.thrown_at;
            // It strikes on its first frame and stands for the rest of the
            // move's active window, so the hit has time to be seen.
            let m = moves::get(p.class, moves::dual::CHAIN_LIGHTNING);
            bolt.life = bolt.life.saturating_add(m.active);
            spawn_effect(&mut self.effects, bolt);
        }
    }

    /// How far along `path` the first thing is -- a body (fighters in versus,
    /// the creature and the critters in a hunt), a wall, a stone, the floor --
    /// or the whole path. The question the ray and the flare both ask.
    fn first_on_the_line(&self, i: usize, path: aim::Path, radius: Fx, terrain: bool) -> Fx {
        let stones = stones::gather(&self.players);
        let seen = self.players;
        let effects = self.effects;
        let scene = aim::Scene {
            stones: &stones,
            players: &seen,
            effects: &effects,
            quarry: &self.monsters,
            critters: &self.critters,
            arena: &self.terrain(),
        };
        let mut targets = aim::Targets::none()
            .fighters(self.pvp())
            .spare(self.allies(i))
            .quarry(true);
        if terrain {
            targets = targets.terrain().stones();
        }
        let full = path.to.sub(path.from).len();
        let met = aim::first_along(path, radius, i as u8, &scene, targets)
            .map_or(full, |c| c.dist().min(full));
        if terrain {
            aim::floor_along(path, &scene).map_or(met, |floor| floor.min(met))
        } else {
            met
        }
    }

    /// **Binary**: the twin stars, raised round her. One pair at a time; a
    /// second Binary renews it rather than adding another.
    fn raise_the_stars(&mut self, i: usize) {
        let p = self.players[i];
        let mut stars = Effect::cast(
            EffectKind::Stars,
            i as u8,
            p.class,
            moves::dual::BINARY,
            p.pos,
            V3::ZERO,
            Fx::ZERO,
        );
        stars.home = V3::new(
            p.pos.x,
            p.pos.y.add(t::body_height().mul(Fx::ratio(1, 2))),
            p.pos.z,
        );
        stars.power = p.thrown_at;
        spawn_the_only_one(&mut self.effects, stars);
    }

    /// **The Shadowstep**: she is behind the first body on the crosshair's line
    /// (`aim::shadowstep_to`) -- further for a cursed one -- and arrives in a
    /// burst that lays its curse, and the airdodge is hers again. With nobody
    /// there she still goes, once a trip: it spends the airdodge, and with it
    /// spent a step at nothing goes nowhere. Paid in aim.
    fn shadowstep(&mut self, i: usize) {
        let p = self.players[i];
        let m = moves::get(p.class, moves::dual::SHADOWSTEP);
        let stones = stones::gather(&self.players);
        let seen = self.players;
        let effects = self.effects;
        let step = {
            let scene = aim::Scene {
                stones: &stones,
                players: &seen,
                effects: &effects,
                quarry: &self.monsters,
                critters: &self.critters,
                arena: &self.terrain(),
            };
            aim::shadowstep_to(
                i,
                p.aim_path.dir(),
                m.reach,
                t::shadowstep_cursed(),
                t::shadowstep_bare(),
                t::shadowstep_behind(),
                self.pvp(),
                &scene,
            )
        };
        // **A step at nothing is once a trip**, as Phase and the airdodge are
        // -- all three spend the airdodge -- and a step to a body refunds it,
        // so stepping from body to body chains as far as there are bodies.
        if step.struck.is_none() && p.air_dodged {
            return;
        }
        let q = &mut self.players[i];
        q.pos = step.to;
        // Arriving at a body she stops there and her fall is caught; with
        // nobody she goes on falling as she was. Reach paid in aim, never
        // free: a Shadowstep at nothing is a step, not a glide.
        let fall = if step.struck.is_some() {
            q.vel.y.max(Fx::ZERO)
        } else {
            q.vel.y
        };
        q.vel = V3::new(Fx::ZERO, fall, Fx::ZERO);
        q.air_dodged = step.struck.is_none();
        if let Some(at) = step.struck {
            // Turned to face what she came for: she arrived behind it.
            let back = V3::new(at.x.sub(step.to.x), Fx::ZERO, at.z.sub(step.to.z));
            if back.flat_len().raw() > 0 {
                q.facing = back.normalized();
            }
            let mut burst = Effect::cast(
                EffectKind::Burst,
                i as u8,
                p.class,
                moves::dual::SHADOWSTEP,
                at,
                V3::ZERO,
                m.radius.mul(p.thrown_at),
            );
            burst.power = p.thrown_at;
            spawn_effect(&mut self.effects, burst);
        }
    }

    /// **Write a sigil** of hers at `at`, `radius` across before her power:
    /// consecrated ground for `sigil, lasts`. At most `sigils, at once` of
    /// hers stand; another replaces the oldest.
    fn write_sigil(&mut self, i: usize, at: V3, radius: Fx, power: Fx) {
        let most = t::sigils_at_once();
        let mine = |e: &Option<Effect>| {
            e.is_some_and(|e| e.kind == EffectKind::Sigil && e.owner == i as u8)
        };
        while self.effects.iter().filter(|e| mine(e)).count() >= most {
            let oldest = self
                .effects
                .iter()
                .enumerate()
                .filter(|(_, e)| mine(e))
                .max_by_key(|(_, e)| e.map_or(0, |e| e.age))
                .map(|(slot, _)| slot);
            match oldest {
                Some(slot) => self.effects[slot] = None,
                None => break,
            }
        }
        let p = self.players[i];
        let kind = if p.action.attack_kind() == Some(moves::dual::SANCTUARY) {
            moves::dual::SANCTUARY
        } else {
            moves::dual::SMITE
        };
        let mut sigil = Effect::cast(
            EffectKind::Sigil,
            i as u8,
            p.class,
            kind,
            at,
            V3::ZERO,
            radius,
        );
        sigil.power = power;
        spawn_effect(&mut self.effects, sigil);
    }

    /// **Judgement's columns**: the middle one is the move's own blow; the
    /// rest -- more the fuller her Light, as `dual::judgement_columns` counts
    /// them -- come down round it in one burst.
    fn bring_down_the_columns(&mut self, i: usize) {
        let p = self.players[i];
        let columns = dual::judgement_columns(p.thrown_at);
        if columns <= 1 {
            return;
        }
        let m = moves::get(p.class, moves::dual::JUDGEMENT);
        let mut burst = Effect::cast(
            EffectKind::Burst,
            i as u8,
            p.class,
            moves::dual::JUDGEMENT,
            p.aim_at(),
            p.facing,
            m.radius.mul(dual::major_size(p.thrown_at)),
        );
        burst.banked = (columns - 1) as i32;
        burst.power = p.thrown_at;
        spawn_effect(&mut self.effects, burst);
    }

    /// **Apotheosis**: the nova that bursts from her as she ascends -- both
    /// forces at the top of her power, `apotheosis, reaches` round her
    /// middle, and every curse inside it purged at full power.
    fn apotheosis(&mut self, i: usize) {
        let p = self.players[i];
        let mid = V3::new(
            p.pos.x,
            p.pos.y.add(t::body_height().mul(Fx::ratio(1, 2))),
            p.pos.z,
        );
        let mut nova = Effect::cast(
            EffectKind::Burst,
            i as u8,
            p.class,
            crate::effects::APOTHEOSIS,
            mid,
            V3::ZERO,
            t::apotheosis_reach(),
        );
        // Its numbers are its own knobs, as they read: what it purges is at
        // the top of the curve (`World::apply_effect`).
        nova.power = Fx::ONE;
        spawn_effect(&mut self.effects, nova);
    }

    /// **The spellbook's frame** (2026-10-12): every curse runs its clock and,
    /// on the tick, deals its stacks and drains them to whoever laid them --
    /// on the fighters and on the creatures -- and every Dual mage learns
    /// whether she is standing in one of her own sigils, which is her ward.
    fn step_spellbook(&mut self) {
        let ticks = dual::ticks_on(self.frame);
        for v in 0..MAX_PLAYERS {
            let curse = self.players[v].curse;
            let n = dual::stacks(curse);
            if n > 0 && ticks && self.players[v].health > 0 {
                let by = dual::cursed_by(curse).min(MAX_PLAYERS - 1);
                let hurt = dual::curse_tick_damage(n, dual::curse_power(&self.players[by]));
                let dealt = self.players[v].wound(hurt);
                if by != v {
                    self.players[by].heal(dealt * t::curse_drains() / 100);
                }
            }
            let q = &mut self.players[v];
            dual::fade_curse(&mut q.curse, &mut q.curse_clock);
        }
        for slot in 0..crate::monster::MAX_MONSTERS {
            let Some(mut beast) = self.monsters[slot] else {
                continue;
            };
            let n = dual::stacks(beast.curse);
            if n > 0 && ticks && beast.alive() {
                let by = dual::cursed_by(beast.curse).min(MAX_PLAYERS - 1);
                let hurt = dual::curse_tick_damage(n, dual::curse_power(&self.players[by]))
                    .min(beast.health)
                    .max(0);
                beast.health -= hurt;
                self.players[by].heal(hurt * t::curse_drains() / 100);
            }
            dual::fade_curse(&mut beast.curse, &mut beast.curse_clock);
            self.monsters[slot] = Some(beast);
        }
        for i in 0..MAX_PLAYERS {
            if self.players[i].class != Class::DualMage {
                continue;
            }
            let feet = self.players[i].pos;
            let warded = self.effects.iter().flatten().any(|e| {
                e.owner == i as u8
                    && matches!(e.kind, EffectKind::Sigil | EffectKind::JudgementField)
                    && {
                        let flat = V3::new(feet.x.sub(e.pos.x), Fx::ZERO, feet.z.sub(e.pos.z));
                        flat.flat_len().raw() <= e.field_radius().raw()
                            && feet.y.sub(e.pos.y).abs().raw() <= t::body_height().raw()
                    }
            });
            dual::set_warded(&mut self.players[i], warded);
        }
    }

    /// Every creature's bleed runs down a frame, and on its tick takes its
    /// damage and spills under it -- the fighter's bleed (`step_bleeds`) on a
    /// body that has no `Player` to keep one.
    fn bleed_the_creatures(&mut self) {
        for slot in 0..crate::monster::MAX_MONSTERS {
            let Some(mut beast) = self.monsters[slot] else {
                continue;
            };
            if beast.bleeding == 0 {
                continue;
            }
            let owner = beast.bled_by as usize;
            if !beast.alive() || owner >= MAX_PLAYERS {
                beast.bleeding = 0;
                self.monsters[slot] = Some(beast);
                continue;
            }
            beast.bleeding -= 1;
            let ticks = beast.bleeding % t::bleed_tick() == 0;
            let tick = self.bleed_tick_of(owner);
            let dealt = if ticks {
                let dealt = tick.min(beast.health).max(0);
                beast.health -= dealt;
                dealt
            } else {
                0
            };
            self.monsters[slot] = Some(beast);
            if dealt > 0 {
                let class = self.players[owner].class;
                let at = V3::new(
                    beast.pos.x,
                    self.terrain().ground_under(beast.pos),
                    beast.pos.z,
                );
                self.spill(owner as u8, class, moves::blood::THORN_LASH, at, dealt);
            }
        }
    }

    /// Raise an Air ball where she pressed for it. One of hers at a time: a
    /// second press while one rolls starts a new one and the old one goes.
    fn raise_the_ball(&mut self, i: usize) {
        let p = self.players[i];
        if self
            .effects
            .iter()
            .flatten()
            .any(|e| e.kind == EffectKind::AirBall && e.owner == i as u8 && !e.ball_sent())
        {
            return;
        }
        let cap = moves::get(p.class, moves::elementalist::AIR_BALL).channel;
        spawn_the_only_one(
            &mut self.effects,
            Effect::cast(
                EffectKind::AirBall,
                i as u8,
                p.class,
                moves::elementalist::AIR_BALL,
                p.ball_at,
                V3::ZERO,
                crate::effects::ball_radius_after(0, cap),
            ),
        );
    }

    /// Let her raised Air ball go, flat along the line the release aimed.
    fn send_the_ball(&mut self, i: usize) {
        let p = self.players[i];
        let dir = p.aim_path.dir();
        let flat = V3::new(dir.x, Fx::ZERO, dir.z);
        if flat.flat_len().raw() == 0 {
            return;
        }
        let flat = flat.normalized();
        for e in self.effects.iter_mut().flatten() {
            if e.kind == EffectKind::AirBall && e.owner == i as u8 && !e.ball_sent() {
                e.dir = flat;
            }
        }
    }

    /// **The earth jump**, on its first active frame, after its lift.
    ///
    /// What it does depends on what she is standing on:
    ///
    /// - **The floor**: a stone comes up with her ([`Structure::brought_up`]),
    ///   a little slower and falling more gently, so a straight jump comes down
    ///   on to it in the air.
    /// - **A stone on the floor**: it shatters outward under her, and the jump
    ///   is bigger (`tuning::earth_shatter_jump`).
    /// - **A stone in the air**: it is driven back into the ground, where it
    ///   shatters, and she jumps off it.
    ///
    /// [`Structure::brought_up`]: crate::class::Structure::brought_up
    fn earth_jump(&mut self, i: usize) {
        let p = self.players[i];
        let feet = p.pos;
        // Off the stone she was standing on when it began, if her feet have
        // only left its edge during the wind-up -- still at its top, and no
        // further past its side than a walk carries her through the wind-up.
        let windup = moves::get(p.class, moves::elementalist::EARTH_JUMP).startup;
        let walked = t::move_speed().mul(DT).mul(Fx::from_int(windup as i32 + 2));
        let last = (p.rise.stone as usize).checked_sub(1).filter(|&index| {
            stones::gather(&self.players)[index].is_some_and(|s| {
                let apart = V3::new(s.at.x.sub(feet.x), Fx::ZERO, s.at.z.sub(feet.z));
                let reach = s.radius().add(t::body_radius()).add(walked);
                apart.flat_len().raw() <= reach.raw()
                    && feet.y.sub(s.top()).abs().raw() <= t::body_radius().raw()
            })
        });
        match stones::under(&self.players, feet).or(last) {
            Some(index) if stones::resting(&self.players, index) => {
                let lit = stones::is_lit(&self.players, index);
                if let Some(middle) = stones::destroy(&mut self.players, index) {
                    debris::shatter(&mut self.debris, i as u8, middle);
                    if lit {
                        spawn_effect(
                            &mut self.effects,
                            Effect::cast(
                                EffectKind::Embers,
                                i as u8,
                                p.class,
                                moves::elementalist::CINDER,
                                middle,
                                V3::ZERO,
                                t::embers_radius(),
                            ),
                        );
                    }
                    let q = &mut self.players[i];
                    q.vel.y = q.vel.y.mul(t::earth_shatter_jump());
                }
            }
            Some(index) => {
                stones::drive_down(&mut self.players, index, t::earth_meteor_speed());
            }
            None => {
                let vel = self.players[i].vel;
                stones::raise(
                    &mut self.players[i],
                    class::Structure::brought_up(feet, vel),
                );
            }
        }
    }

    /// **One frame of an Air ball**: grow while she holds it, or roll, shrink
    /// and carry once it is sent.
    ///
    /// Raised, it sits where the crosshair put it and its radius follows the
    /// hold (`effects::ball_radius_after`); knocked out of the hold, it comes
    /// apart where it stands. Sent, it rolls along the floor at the speed its
    /// size gives it (`effects::ball_speed`), shrinking at the steady rate --
    /// faster while it carries somebody -- and is gone when it has shrunk
    /// away. A wall or a stone turns it and costs it size; an edge it rolls
    /// off, and sinks.
    ///
    /// **Carrying** is a moving floor: a body standing in it is moved with it,
    /// frame for frame, and can still walk inside it; a body that jumps inside
    /// it is given its speed, so it leaves with it. A ball smaller than
    /// `tuning::air_ball_holds` holds nobody, so the rider is put down a
    /// moment before the ball vanishes rather than dropped when it does.
    /// The Fire carpet comes down while it hangs (2026-10-21, from play), at
    /// `tuning::carpet_sink`, each end on its own: an end that reaches the
    /// floor under it rests there, its capsule's radius up, and the other
    /// keeps coming until it does too. So a carpet laid low enough ends up
    /// lying on the ground, along the slope it landed on, and one laid high
    /// is still in the air when it burns out. The floor is
    /// `Terrain::floor_below`, so a stone or a ledge under an end catches it.
    fn sink_the_carpet(&self, effect: &mut Effect) {
        let drop = t::carpet_sink().mul(DT);
        if drop == Fx::ZERO {
            return;
        }
        let terrain = self.terrain();
        let rest = effect.field_radius();
        let lower = |end: V3| {
            let floor = terrain.floor_below(end).add(rest);
            let y = end.y.sub(drop).max(floor).min(end.y);
            V3::new(end.x, y, end.z)
        };
        let (a, b) = effect.carpet_line();
        let (near, far) = (lower(a), lower(b));
        if near.y == a.y.sub(drop) && far.y == b.y.sub(drop) {
            // Both ends still falling: the strip comes down whole, its line
            // as it was laid.
            effect.pos = near;
            return;
        }
        if near == a && far == b {
            return;
        }
        // An end has landed: the strip lies between where its ends are now.
        let run = far.sub(near);
        effect.pos = near;
        effect.reach = run.len();
        if effect.reach.raw() > 0 {
            effect.dir = run.normalized();
        }
    }

    fn roll_the_ball(&mut self, effect: &mut Effect) {
        use moves::elementalist::AIR_BALL;
        let owner = effect.owner as usize;
        if !effect.ball_sent() {
            let p = self.players[owner];
            match p.action {
                Action::Channel { kind, held } if kind == AIR_BALL => {
                    let cap = moves::get(p.class, kind).channel;
                    effect.reach = crate::effects::ball_radius_after(held, cap);
                }
                // Let go, and about to be sent on the move's first active
                // frame: it holds the size it was let go at.
                Action::Startup { kind, .. } | Action::Active { kind, .. } if kind == AIR_BALL => {}
                _ => effect.reach = Fx::ZERO,
            }
            return;
        }

        let speed = crate::effects::ball_speed(effect.reach);
        let step = effect.dir.scale(speed.mul(DT));
        // A ball climbs a step up to half its own size.
        let climbs = effect.reach.mul(Fx::ratio(1, 2));
        let terrain = self.terrain();
        let next = effect.pos.add(step);
        let probe = V3::new(next.x, next.y.add(climbs), next.z);
        let r = terrain.resolve_sized(
            probe,
            step.scale(Fx::ONE.div(DT)),
            true,
            effect.reach,
            effect.reach.add(effect.reach),
        );
        // What it ran into, as the way back out of it: the side of a stone,
        // or the wall the arena pushed it off.
        let stone_in_the_way = stones::gather(&self.players)
            .iter()
            .flatten()
            .find_map(|stone| {
                let apart = V3::new(next.x.sub(stone.at.x), Fx::ZERO, next.z.sub(stone.at.z));
                let meets = apart.flat_len().raw()
                    < stone.radius().add(effect.reach.mul(Fx::ratio(1, 2))).raw()
                    && stone.top().raw() > next.y.add(climbs).raw()
                    && stone.at.y.raw() < next.y.add(effect.reach).raw();
                meets.then(|| apart.normalized())
            });
        let knocked = stone_in_the_way.or_else(|| {
            r.wall.then(|| {
                let out = V3::new(r.pos.x.sub(probe.x), Fx::ZERO, r.pos.z.sub(probe.z));
                out.normalized()
            })
        });
        let new_pos = if let Some(out) = knocked {
            // **Knocked off, not stopped** (2026-10-09, from play): it turns
            // off the surface it met, as a ball off a wall, and loses some of
            // its size -- so its speed and what is left of its life -- and
            // stays where it was this frame.
            let out = if out == V3::ZERO {
                V3::ZERO.sub(effect.dir)
            } else {
                out
            };
            let into = effect.dir.x.mul(out.x).add(effect.dir.z.mul(out.z));
            if into.raw() < 0 {
                let back = out.scale(into.add(into));
                let turned = V3::new(effect.dir.x.sub(back.x), Fx::ZERO, effect.dir.z.sub(back.z));
                effect.dir = turned.normalized();
            }
            effect.reach = effect.reach.mul(t::air_ball_knock());
            effect.pos
        } else {
            // **Off an edge it sinks** rather than dropping (2026-10-09, from
            // play): over a floor no further down than a slope as steep as
            // its own frame's roll, it rolls on that floor; past an edge it
            // keeps going and comes down slowly, at `tuning::air_ball_sink`,
            // carrying whoever is in it, until it meets the floor below.
            let floor = terrain.floor_below(V3::new(r.pos.x, r.pos.y.add(effect.reach), r.pos.z));
            let slope = step.flat_len();
            let y = if floor.raw() >= effect.pos.y.sub(slope).raw() {
                floor
            } else {
                effect.pos.y.sub(t::air_ball_sink().mul(DT)).max(floor)
            };
            V3::new(r.pos.x, y, r.pos.z)
        };
        let moved = new_pos.sub(effect.pos);
        effect.pos = new_pos;

        let holds = effect.reach.raw() >= t::air_ball_holds().raw();
        let mut carrying = false;
        for i in 0..MAX_PLAYERS {
            let p = self.players[i];
            let apart = V3::new(
                p.pos.x.sub(effect.pos.x),
                Fx::ZERO,
                p.pos.z.sub(effect.pos.z),
            );
            let in_it = holds
                && p.health > 0
                && apart.flat_len().raw() < effect.reach.raw()
                && p.pos.y.raw() >= effect.pos.y.sub(climbs).raw()
                && p.pos.y.raw() < effect.pos.y.add(effect.reach).add(effect.reach).raw();
            if !in_it {
                effect.struck.unset(crate::effects::Effect::carried_bit(i));
                continue;
            }
            // Whoever it has hold of -- her, or anybody it rolled over -- goes
            // with it. Getting out is jumping out, or walking out the side.
            effect.take_hit(0, i);
            carrying = true;
            let q = &mut self.players[i];
            // Held up by it where there is no floor: off an edge, a body
            // standing in it stands on it, and sinks with it.
            let held = !q.grounded
                && q.vel.y.raw() <= 0
                && q.pos.y.raw() <= effect.pos.y.add(climbs).raw();
            if q.grounded || held {
                q.pos.x = q.pos.x.add(moved.x);
                q.pos.z = q.pos.z.add(moved.z);
                if held {
                    q.pos.y = effect.pos.y;
                    q.vel.y = Fx::ZERO;
                    q.grounded = true;
                }
            } else {
                q.vel.x = effect.dir.x.mul(speed);
                q.vel.z = effect.dir.z.mul(speed);
            }
        }
        let mut shrink = t::air_ball_shrink().mul(DT);
        if carrying {
            shrink = shrink.mul(t::air_ball_carried_shrink());
        }
        effect.reach = effect.reach.sub(shrink).max(Fx::ZERO);

        // **Rolled through fire, it is fire** (2026-10-18): it burns whoever
        // it holds on the tick -- the one wind move that hurt nobody -- and
        // lights a stone it runs into.
        let (radius, tall) = (effect.reach, effect.reach.add(effect.reach));
        if !effect.lit() && self.touches_fire(effect.pos, radius, tall) {
            effect.catch_fire();
        }
        if effect.lit() && effect.reach.raw() > 0 {
            stones::light_touching(&mut self.players, effect.pos, radius, tall);
            if effect.ticks_now() {
                self.burn_in_column(effect.owner, effect.pos, radius, tall, t::wind_fire_burn());
            }
        }
    }

    /// **Is this column touching fire?** `radius` wide, from `foot` up
    /// `height`: any fire effect in its own shape (`body_in_fire`), or a lit
    /// stone. What sets wind alight (`Effect::lit`).
    fn touches_fire(&self, foot: V3, radius: Fx, height: Fx) -> bool {
        if self
            .effects
            .iter()
            .flatten()
            .any(|e| e.kind.ignites() && body_in_fire(e, foot, radius, height))
        {
            return true;
        }
        stones::gather(&self.players).iter().flatten().any(|s| {
            let apart = V3::new(s.at.x.sub(foot.x), Fx::ZERO, s.at.z.sub(foot.z));
            s.lit > 0
                && crate::math::wide_flat_len(apart).raw() <= radius.add(s.radius()).raw()
                && foot.y.raw() <= s.top().raw()
                && foot.y.add(height).raw() >= s.at.y.raw()
        })
    }

    /// **Burn everything standing in a column** but its owner -- fighters,
    /// creatures and critters -- for `damage`, as fire under a body does
    /// (no stun, no shove). Burning wind's bite.
    fn burn_in_column(&mut self, owner: u8, foot: V3, radius: Fx, height: Fx, damage: i32) {
        let column = crate::effects::Pillar {
            radius,
            bottom: Fx::ZERO,
            top: height,
        };
        for i in 0..MAX_PLAYERS {
            let p = self.players[i];
            if self.effects_reach(i, owner)
                && column.contains(foot, p.pos, t::body_radius(), t::body_height())
            {
                self.players[i].wound(damage);
            }
        }
        for beast in self.monsters.iter_mut().flatten() {
            if beast.alive() && beast.part_struck(foot, radius, height).is_some() {
                beast.scorch(damage);
            }
        }
        for i in 0..crate::critter::MAX_CRITTERS {
            let c = self.critters[i];
            if c.alive() && column.contains(foot, c.pos, Fx::ZERO, Fx::ONE) {
                pack::hurt(
                    &mut self.pack,
                    &mut self.critters,
                    i,
                    damage,
                    V3::ZERO,
                    Fx::ZERO,
                    Fx::ZERO,
                );
            }
        }
    }

    /// A draft that has touched fire burns each body in it once, on its own
    /// part (2), and is drawn as fire from then on.
    fn burn_in_the_draft(&mut self, effect: &mut Effect) {
        let slab = effect.draft_volume();
        let foot = V3::new(effect.pos.x, effect.pos.y.add(slab.bottom), effect.pos.z);
        let tall = slab.top.sub(slab.bottom);
        if !effect.lit() && self.touches_fire(foot, slab.radius, tall) {
            effect.catch_fire();
        }
        if !effect.lit() {
            return;
        }
        stones::light_touching(&mut self.players, foot, slab.radius, tall);
        for i in 0..MAX_PLAYERS {
            let p = self.players[i];
            if self.effects_reach(i, effect.owner)
                && !effect.already_hit(2, i)
                && slab.contains(effect.pos, p.pos, t::body_radius(), t::body_height())
            {
                effect.take_hit(2, i);
                self.players[i].wound(t::draft_fire_burn());
            }
        }
        // Creatures and critters on the column's first burning frame only:
        // they have no bit of their own here.
        if !effect.already_hit(2, effect.owner as usize) {
            effect.take_hit(2, effect.owner as usize);
            for beast in self.monsters.iter_mut().flatten() {
                if beast.alive() && beast.part_struck(foot, slab.radius, tall).is_some() {
                    beast.scorch(t::draft_fire_burn());
                }
            }
        }
    }

    /// **She steers the Air ball she is riding** (2026-10-18, from play: the
    /// first steering, by the sideways share of her walk, walked her out of
    /// the side of it before it turned at all). While she is carried by her
    /// own ball, A and D turn it -- left and right, at
    /// `tuning::air_ball_turn` -- and her walk is taken off her, so she stays
    /// in it however she steers. A jump still leaves it. Anybody else it
    /// carries walks as they like.
    fn steer_the_balls(&mut self, mut wire: [Input; MAX_PLAYERS]) -> [Input; MAX_PLAYERS] {
        for e in self.effects.iter_mut().flatten() {
            if e.kind != EffectKind::AirBall || !e.ball_sent() {
                continue;
            }
            let owner = e.owner as usize;
            if owner >= MAX_PLAYERS || !e.already_hit(0, owner) || !self.players[owner].grounded {
                continue;
            }
            let input = &mut wire[owner];
            let (ax, _) = input.move_axis();
            if ax != 0 {
                let yaw = crate::math::atan2_turns(e.dir.z, e.dir.x);
                let turn = t::air_ball_turn().mul(DT).mul(Fx::from_int(ax));
                e.dir = V3::from_turns(yaw.add(turn));
            }
            input.bits &= !(Input::W | Input::A | Input::S | Input::D);
        }
        wire
    }

    /// **The Thermal**: her Updraft and her own fire sharing space, whichever
    /// came first.
    ///
    /// Asked every frame the column blows, so it does not matter whether she
    /// took off in fire or laid the carpet into a column already blowing. Once
    /// per column, on the owner's own bit. It throws her up harder than the
    /// Updraft does, and if the fire is a Fire carpet, along the carpet too --
    /// and the carpet is used up, because the heat went with her.
    fn ride_the_thermal(&mut self, effect: &mut Effect) {
        let owner = effect.owner as usize;
        if effect.already_hit(1, owner) {
            return;
        }
        let p = self.players[owner];
        if p.health <= 0
            || !effect.draft_volume().contains(
                effect.pos,
                p.pos,
                t::body_radius(),
                t::body_height(),
            )
        {
            return;
        }
        let (radius, height) = (t::body_radius(), t::body_height());
        // The column's own axis, for the carpet: a strip that reaches into
        // the column anywhere along its height is in it.
        let foot = effect.pos;
        let crown = V3::new(foot.x, foot.y.add(t::draft_height()), foot.z);
        let mut fire = None;
        for slot in 0..MAX_EFFECTS {
            let Some(e) = self.effects[slot] else {
                continue;
            };
            if e.owner != effect.owner {
                continue;
            }
            let meets = match e.kind {
                EffectKind::FireCarpet => {
                    let (a, b) = e.carpet_line();
                    crate::math::segment_gap(a, b, foot, crown).raw()
                        <= effect.reach.add(e.field_radius()).raw()
                }
                _ => body_in_fire(&e, p.pos, radius, height),
            };
            if meets {
                fire = Some(slot);
                if e.kind == EffectKind::FireCarpet {
                    break;
                }
            }
        }
        let Some(slot) = fire else {
            return;
        };
        effect.take_hit(1, owner);
        let q = &mut self.players[owner];
        q.vel.y = q.vel.y.max(t::thermal_lift());
        q.grounded = false;
        q.fall_over = Fx::ZERO;
        if let Some(carpet) = self.effects[slot].filter(|e| e.kind == EffectKind::FireCarpet) {
            let along = carpet.dir.scale(t::thermal_push());
            q.vel.x = q.vel.x.add(along.x);
            q.vel.z = q.vel.z.add(along.z);
            if along.y.raw() > 0 {
                q.vel.y = q.vel.y.add(along.y);
            }
            self.effects[slot] = None;
        }
    }
}

/// The same shape of question `bolt::throws_a_beam` asks, and for the same
/// reason: being a skillshot decides how the move is aimed, not what it does
/// when it lands, so the slot has to be checked too. See `moves::SLOT_HEAVY`.
fn throws_a_cataclysm(p: &Player, kind: u8) -> bool {
    p.class == Class::Elementalist
        && kind == SLOT_HEAVY
        && moves::get(p.class, kind).aim() == aim::Kind::Skillshot
}

// ---------------------------------------------------------------------------
// Trading with the creature
// ---------------------------------------------------------------------------

impl World {
    /// Both directions of the exchange, once both sides have moved.
    fn trade_with_the_creature(&mut self) {
        for slot in 0..MAX_MONSTERS {
            self.trade_with(slot);
        }
    }

    /// **Press a move's button**, for the tools that measure moves rather than
    /// play them (`crate::critcheck`): fighter `who` starts move `kind` this
    /// instant, aimed by `look`, through the same `begin_move` a press goes
    /// through -- the same aim, the same costs -- without having to know which
    /// keys a class binds it to.
    pub fn press(&mut self, who: usize, kind: u8, look: Input) {
        let field = stones::gather(&self.players);
        let seen = self.players;
        let effects = self.effects;
        let beasts = self.monsters;
        let crowd = self.critters;
        let scene = Scene {
            stones: &field,
            players: &seen,
            effects: &effects,
            quarry: &beasts,
            critters: &crowd,
            arena: &self.terrain(),
        };
        let p = &mut self.players[who.min(MAX_PLAYERS - 1)];
        let aerial = !p.grounded;
        p.action = begin_move(p, who, kind, look, &scene, aerial);
        // A Dual mage's long hold is thrown as the hold come full: the press
        // asked for the spell, not for a click that might be a tap.
        if moves::charge(p.class, kind) == Some(moves::Charge::TapOrHold) {
            lock_aim(p, who, kind, look, &scene);
            p.action = throw_move(p, kind, look, aerial);
        }
    }

    /// **A thrown or recalled shield strikes the creatures and the critters
    /// it meets** (`CLASS-5` in `docs/design/review.md`), as it strikes a
    /// fighter in versus: thrown, the first it meets takes `bulwark::throw_damage`
    /// and the shield plants there empty, having spent what it carried; on the
    /// way home it deals `shield_damage` to each it passes through, once each,
    /// and goes on (`kits/bulwark.md`, *Throw / Recall*).
    ///
    /// **The swing's own path, not a second one.** The volume is
    /// [`shield_hitbox`], the overlay's; a creature's part is found by
    /// [`part_under`] and struck through `Monster::take_blow`, so its guard,
    /// its weak points, its breakable parts and whether a part is there at all
    /// (a buried Sandmaw, a cloaked Veilstalker) are its own rules, as they are
    /// for a sword. A critter is touched by `critter::Body::touched_by`, asked
    /// of its guard (`PackMind::guarded`) and struck by `pack::struck`.
    ///
    /// Within one frame's stretch -- a third of a metre at most -- a critter is
    /// met before the creature: a body standing in front of a leg, or riding
    /// it, is what a shield flying at the leg reaches first.
    fn shields_strike_the_quarry(&mut self) {
        for owner in 0..MAX_PLAYERS {
            let thrower = self.players[owner];
            let Some(Shield::Flying {
                pos,
                vel,
                outbound,
                travelled,
                weight,
                struck,
                met,
            }) = thrower.shield()
            else {
                continue;
            };
            let Some(volume) = shield_hitbox(&thrower) else {
                continue;
            };
            if thrower.health <= 0 {
                continue;
            }
            let damage = if outbound {
                bulwark::throw_damage(weight)
            } else {
                t::shield_damage()
            };
            let along = V3::new(vel.x, Fx::ZERO, vel.z).normalized();
            // **Where the blow comes from**, for a guard to ask: back along
            // its flight by as far as it has flown, on the floor -- where it
            // was thrown from, or where it was planted. A guard reads the
            // point a blow is delivered from as a body's feet (the Mantis's
            // asks whether it is under its own belly), and the shield is
            // delivered from there, not from the place it touched.
            let came = pos.sub(along.scale(travelled));
            let from = V3::new(came.x, self.terrain().ground_under(came), came.z);
            let mut met = met;
            let mut hit = false;

            // The small bodies first.
            if let Some(mut brain) = self.pack {
                let sp = brain.sp();
                for c in 0..critter::MAX_CRITTERS {
                    if hit && outbound {
                        break;
                    }
                    let body = self.critters[c];
                    if !body.alive() || met.critters.has(c) || !body.body(sp).touched_by(&volume) {
                        continue;
                    }
                    met.critters.set(c);
                    hit = true;
                    let guarded = match sp.pack {
                        Some(decl) => decl.mind.guarded(
                            &mut brain,
                            &mut self.critters,
                            c,
                            &pack::Blow {
                                who: owner,
                                from,
                                hitbox: &volume,
                                damage,
                                breaks: false,
                            },
                        ),
                        None => pack::Guarded::Lands,
                    };
                    // Turned: nothing reaches the body. The thrower is metres
                    // away, so there is nobody for the recoil to stop.
                    if matches!(guarded, pack::Guarded::Bounces { .. }) {
                        continue;
                    }
                    pack::struck(
                        &mut brain,
                        &mut self.critters,
                        c,
                        damage,
                        along,
                        t::shield_knockback(),
                        Fx::ZERO,
                    );
                    // Loaded, it knocks a small body down as it would a
                    // fighter, for the frames a fighter would lie there --
                    // **if the blow knocked it out of what it was doing at
                    // all**, which is its kind's own rule (`FlinchAt`): a bull
                    // that shrugs the blow off is not lying on the floor.
                    let body = &mut self.critters[c];
                    if outbound
                        && bulwark::knocks_down(weight)
                        && body.alive()
                        && body.state == critter::is::FLINCH
                    {
                        body.timer = body.timer.max(t::knockdown_frames());
                    }
                }
                self.pack = Some(brain);
            }

            // Then the creatures.
            for slot in 0..MAX_MONSTERS {
                if hit && outbound {
                    break;
                }
                let Some(mut beast) = self.monsters[slot] else {
                    continue;
                };
                if !beast.alive() || met.creatures.has(slot) {
                    continue;
                }
                let Some(part) = part_under(&beast, &thrower, &volume) else {
                    continue;
                };
                met.creatures.set(slot);
                hit = true;
                // Its guard is asked as it is of any blow: blocked, parried
                // or broken is the creature's to say, and a blocked throw has
                // still struck something.
                beast.take_blow(
                    part,
                    damage,
                    &monster::Blow {
                        from,
                        unblockable: false,
                        who: owner as u8,
                        class: thrower.class,
                        kind: monster::Blow::NO_MOVE,
                    },
                );
                self.monsters[slot] = Some(beast);
            }

            if !hit {
                continue;
            }
            // Thrown, it plants where it struck, empty -- what it carried
            // went in. Recalled, it goes on home, remembering whom it has
            // been through.
            self.players[owner].mechanic = Mechanic::Shield(if outbound {
                Shield::Planted {
                    pos: V3::new(pos.x, self.terrain().ground_under(pos), pos.z),
                    weight: Fx::ZERO,
                }
            } else {
                Shield::Flying {
                    pos,
                    vel,
                    outbound,
                    travelled,
                    weight,
                    struck,
                    met,
                }
            });
        }
    }

    /// Both directions of the exchange with the pack's small bodies.
    ///
    /// **A fighter's swing is tested against every critter's box by
    /// [`critter::Body::touched_by`]**, the volume being [`hitbox`] -- the one
    /// description of an attack the overlay draws -- and each body it touches
    /// is struck **once per swing**, not once per frame and not only the
    /// first: a sweep through a heap catches the heap. The memory of who has
    /// been struck is on the critter (`flag::STRUCK`), cleared when the swing
    /// ends or a multi-hit rearms, so the creature's `hit_used` rule (one
    /// connection per swing) is left as it was.
    fn trade_with_the_pack(&mut self) {
        let Some(mut brain) = self.pack else { return };
        let sp = brain.sp();

        // What the critters have out. Every fighter in a volume is hit.
        for c in 0..critter::MAX_CRITTERS {
            let body = self.critters[c];
            if !body.alive()
                || body.state != critter::is::ACTIVE
                || body.has(critter::flag::HIT_USED)
            {
                continue;
            }
            let m = sp.attack(body.act);
            let Some((anchor, _, _, _)) = body.hit_volume(sp) else {
                continue;
            };
            let mut landed = false;
            for i in 0..MAX_PLAYERS {
                let victim = self.players[i];
                if victim.health <= 0 || victim.untouchable() {
                    continue;
                }
                if !body.reaches(sp, victim.pos, victim.hurt_height(), t::body_radius()) {
                    continue;
                }
                if sp
                    .pack
                    .is_some_and(|decl| decl.mind.spares(&brain, &self.critters, c, i))
                {
                    continue;
                }
                // Begun off their screen, and no marker under them for a
                // reaction: no tell, so no hit (`pack::watch`).
                if pack::unanswered(&brain, c, i) {
                    continue;
                }
                let away = V3::new(
                    victim.pos.x.sub(anchor.x),
                    Fx::ZERO,
                    victim.pos.z.sub(anchor.z),
                )
                .normalized();
                let away = if away.flat_len().raw() == 0 {
                    body.facing()
                } else {
                    away
                };
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
                        grabs: m.root,
                        by: QUARRY,
                        dir: away,
                        blocked: guarding,
                        parried,
                        interrupts: true,
                    },
                );
                if parried {
                    // A parried critter is knocked out of it, as any hit would.
                    let flinch = critter::stat(sp, body.kind, critter::CritterField::FlinchFrames);
                    let hit = &mut self.critters[c];
                    hit.state = critter::is::FLINCH;
                    hit.timer = (t::parry_stagger() as i32).max(flinch) as u16;
                }
                // Met by a Champion's heavy blow (`meets_a_clash`): to the
                // pack's mind it was turned, as a guard turns it.
                let clashed = matches!(
                    self.players[i].mechanic,
                    Mechanic::Forms { fury, .. } if fury.clash_by != NOBODY
                );
                let guarding = guarding || clashed;
                if !parried && !guarding {
                    let now = self.monsters;
                    fall_off(&mut self.players[i], &now);
                }
                if let Some(decl) = sp.pack {
                    decl.mind.landed(
                        &mut brain,
                        &mut self.critters,
                        c,
                        i,
                        &mut self.players[i],
                        guarding,
                        parried,
                    );
                }
                freeze(
                    &mut self.players[i],
                    impact_freeze(t::creature_freeze(), guarding && !parried),
                );
                landed = true;
            }
            if landed {
                self.critters[c].set(critter::flag::HIT_USED, true);
            }
        }
        // And the defended things in their reach (P7), once a move: the
        // Hornback's bull charging the cart.
        self.critters_strike_the_objectives();

        // What the fighters have out.
        for i in 0..MAX_PLAYERS {
            let attacker = self.players[i];
            let swing = hitbox(&attacker).filter(|_| attacker.health > 0);
            // A new swing, or one that has rearmed: nobody has been struck by
            // it yet.
            if swing.is_none() || !attacker.hit_used {
                for c in self.critters.iter_mut() {
                    c.swung.unset(i);
                }
            }
            let Some(box_out) = swing else { continue };
            let Some(kind) = attacker.action.attack_kind() else {
                continue;
            };
            // The beam and the heavy already had their answers, on the frame
            // they fired, through `aim::first_along`.
            if bolt::throws_a_beam(&attacker, kind) || throws_a_cataclysm(&attacker, kind) {
                continue;
            }
            let m = moves::get(attacker.class, kind);
            let worth = Fx::from_int(
                bulwark::slam_damage(&attacker, kind, m.damage) + strike_bonus(&attacker, kind),
            )
            .mul(preying(attacker.class, false))
            .mul(grey_power(&attacker, &m))
            .mul(blood_blow(&attacker, kind))
            .mul(depth(&attacker))
            .mul(if box_out.sweet {
                t::orb_sweet()
            } else {
                Fx::ONE
            })
            .to_int();
            // A guard breaker, by the versus rule's own flag -- or a Bulwark
            // Slam carrying weight, which is what a loaded shield is for.
            let breaks = m.unblockable
                || (attacker.class == Class::Bulwark
                    && kind == SLOT_COMMITTED
                    && bulwark::weight(&attacker).raw() > 0);
            let mut any = false;
            for c in 0..critter::MAX_CRITTERS {
                let body = self.critters[c];
                if !body.alive() || body.swung.has(i) || !body.body(sp).touched_by(&box_out) {
                    continue;
                }
                self.critters[c].swung.set(i);
                // **A guard, if the body has one** (`PackMind::guarded`): a
                // turned blow does nothing to it, and the swing recoils.
                let guarded = match sp.pack {
                    Some(decl) => decl.mind.guarded(
                        &mut brain,
                        &mut self.critters,
                        c,
                        &pack::Blow {
                            who: i,
                            from: attacker.pos,
                            hitbox: &box_out,
                            damage: worth,
                            breaks,
                        },
                    ),
                    None => pack::Guarded::Lands,
                };
                if let pack::Guarded::Bounces { recoil, push } = guarded {
                    let back = V3::new(
                        attacker.pos.x.sub(body.pos.x),
                        Fx::ZERO,
                        attacker.pos.z.sub(body.pos.z),
                    )
                    .normalized();
                    let p = &mut self.players[i];
                    p.action = Action::HitStun { left: recoil };
                    p.stun_total = recoil;
                    // Pushed back the distance asked, over the recoil: a
                    // blocker's pushback, the other way round.
                    let rate =
                        Fx::from_int(crate::TICK_HZ as i32).div(Fx::from_int(recoil.max(1) as i32));
                    p.vel.x = back.x.mul(push).mul(rate);
                    p.vel.z = back.z.mul(push).mul(rate);
                    freeze(p, impact_freeze(t::creature_freeze(), true));
                    any = true;
                    continue;
                }
                let dir = if m.knockback.raw() < 0 {
                    attacker.pos.sub(body.pos)
                } else {
                    body.pos.sub(attacker.pos)
                };
                let dir = V3::new(dir.x, Fx::ZERO, dir.z).normalized();
                let before = body.health as i32;
                pack::struck(
                    &mut brain,
                    &mut self.critters,
                    c,
                    worth,
                    dir,
                    m.knockback.abs(),
                    m.launch.max(Fx::ZERO),
                );
                let dealt = before.min(worth).max(0);
                // A link that lands on a critter -- the Hornback's bull, a
                // gnawer -- is a landed link like any other: it confirms the
                // string and feeds his momentum (exploration 0015).
                confirm_chain(&mut self.players[i], dealt <= 0);
                if dealt > 0 {
                    feed_the_momentum(&mut self.players[i], Some(kind));
                }
                if dealt > 0 {
                    self.drink_over(i, &m, box_out.from, box_out.to, None, 0);
                    self.spill(
                        i as u8,
                        attacker.class,
                        kind,
                        floor_under(self.arena(), body.pos),
                        dealt.max(worth),
                    );
                    self.players[i].heal(m.leeched(dealt));
                    dual::landed_a_hit(&mut self.players[i]);
                }
                any = true;
            }
            if any {
                self.players[i].hit_used = true;
            }
        }
        self.pack = Some(brain);
    }

    /// Both directions of the exchange with the creature in one slot.
    fn trade_with(&mut self, slot: usize) {
        let Some(mut beast) = self.monsters[slot] else {
            return;
        };
        // The herd as it stands, with this creature as it stands *now*: who
        // falls off what reads the pose it has at the moment they fall.
        let herd = |beast: &Monster, monsters: &Herd| {
            let mut herd = *monsters;
            herd[slot] = Some(*beast);
            herd
        };

        // What the creature has out. Every fighter it reaches is hit on the
        // same frame -- spending the hit on whoever happened to be checked
        // first would make a coop partner a shield.
        // Who it landed on, and whether they had it on a guard: handed to the
        // species once the exchange is over (`FightDecl::landed`).
        let mut struck = [None; MAX_PLAYERS];
        if let (Doing::Active { kind, .. }, false) = (beast.doing, beast.hit_used) {
            let m = beast.sp().attack(kind);
            let anchor = beast
                .hit_volume()
                .map(|(a, _, _, _)| a)
                .unwrap_or(beast.pos);
            let mut landed = false;
            for (i, struck_one) in struck.iter_mut().enumerate() {
                let victim = self.players[i];
                if victim.health <= 0 || victim.untouchable() {
                    continue;
                }
                // Nothing it swings reaches somebody it has swallowed.
                if inside(&victim, &self.monsters) {
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
                // **The Clash** (exploration 0015, C7): its swing met a
                // Champion's heavy blow in the middle of being thrown. The
                // swing is cancelled -- the creature is knocked out of it --
                // and he takes nothing. Hitting the scythe, rather than
                // getting out of its way.
                if clashing(&victim) {
                    beast.doing = Doing::Flinch {
                        left: t::clash_flinch(),
                    };
                    let q = &mut self.players[i];
                    freeze(q, t::clash_freeze());
                    q.parried = PARRY_FLOURISH;
                    feed_the_momentum(q, None);
                    landed = true;
                    *struck_one = Some((kind, true));
                    continue;
                }
                apply_hit(
                    &mut self.players[i],
                    Hit {
                        damage: m.damage,
                        hitstun: m.hitstun,
                        blockstun: m.blockstun,
                        knockback: m.knockback,
                        launch: m.launch,
                        // A rooting hit holds the victim where they stand,
                        // by nobody: `drag_the_held` hauls only toward a
                        // fighter, so a hold by the creature is a pin.
                        grabs: m.root,
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
                    let now = herd(&beast, &self.monsters);
                    fall_off(&mut self.players[i], &now);
                }
                // The victim's half of an impact freeze. The animal does not
                // stop for one fighter; see `tuning::creature_freeze`.
                freeze(
                    &mut self.players[i],
                    impact_freeze(t::creature_freeze(), guarding && !parried),
                );
                landed = true;
                *struck_one = Some((kind, guarding || parried));
            }
            if landed {
                beast.hit_used = true;
            }
        }
        // And the defended things in its reach (P7), once a move.
        self.strike_the_objectives(slot, &beast);

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
            // The Reaver's cash-in, on a leg rather than a person: a finisher
            // of hers spends the creature's marks, and it cannot block.
            // What a full tally does to something this size is its own flinch
            // and poise rules' business -- a hit three times the size is
            // already the thing those read.
            let finisher = attacker.class == Class::ShadowReaver && moves::reaver::cashes(kind);
            let full = finisher && shadow::full_tally(beast.marks);
            let cash = if finisher {
                shadow::cash_multiple(beast.marks)
            } else {
                Fx::ONE
            };
            let sweet = reave_sweet(&attacker, kind, &box_out, |c, r| {
                beast.part_struck(c, r, attacker.hurt_height()).is_some()
            });
            let raw = Fx::from_int(bulwark::slam_damage(&attacker, kind, m.damage))
                .mul(preying(attacker.class, beast.disabled()))
                .mul(cash)
                .mul(sweet)
                .mul(true_link_mul(&attacker))
                .mul(Fx::ONE.add(spent_share_of(&attacker, kind)))
                .to_int();
            // Her double duty, against the creature: drink the stain the
            // blade passes through, and the blow is harder for it. Before
            // the blow, so the creature takes the whole.
            let raw = raw + self.stain_bonus(i, &m, box_out.from, box_out.to, None, 0);
            // Asked of its guard first, from where the swing comes from:
            // the same point the fighters' guard is handed.
            let (dealt, guarded) = beast.take_blow(
                part,
                raw,
                &monster::Blow {
                    from: attacker.pos,
                    unblockable: m.unblockable,
                    who: i as u8,
                    class: attacker.class,
                    kind,
                },
            );
            rebound_off_a_body(&mut self.players[i]);
            // A chain link that lands on the animal is a hit confirm like one
            // that lands on a fighter: its recovery may be cut short for the
            // next, and a third gives the Rush back. One its guard turned is a
            // blocked link and pays in full. (Until 2026-10-12 nothing here
            // confirmed the chain, so every string against a creature paid
            // every recovery in full.)
            // The momentum he carries hurts it more and weighs nothing: its
            // share goes on after the blow, past the strain (`take_extra`).
            let dealt = dealt
                + beast.take_extra(Fx::from_int(dealt).mul(carried_share(&attacker)).to_int());
            confirm_chain(&mut self.players[i], guarded.turned() || dealt <= 0);
            if dealt > 0 && !guarded.turned() {
                freeze_on_the_creature(&mut self.players[i], &mut beast, &attacker, &m);
            }
            if dealt > 0 {
                feed_the_momentum(&mut self.players[i], Some(kind));
                impact(&mut beast, &attacker, kind, dealt);
            }
            // The marks are spent by a blow that landed; one its guard
            // turned spends nothing.
            if finisher && !guarded.turned() {
                beast.spend_marks();
                // On the creature, the Reaping is round the place the blow
                // met, outside the part it met: a leg, a flank, the whole
                // back she is standing on.
                if full {
                    let met = box_out.centre();
                    let floor = floor_under(self.arena(), met);
                    let thick = beast.part_reach_from(part, met);
                    reap(&mut self.effects, &attacker, i, met, floor, thick);
                }
            }
            // Her double duty, against the creature: drink over the pool the
            // blade passes through, then spill under the part it struck.
            if dealt > 0 || (raw > 0 && !guarded.turned()) {
                self.spill(
                    i as u8,
                    attacker.class,
                    kind,
                    floor_under(self.arena(), box_out.centre()),
                    dealt.max(raw),
                );
            }
            // An uppercut does not put thirteen metres of animal in the air and
            // a grab does not drag it anywhere, but throwing one at a creature
            // that is already reeling should still be a decision -- so the move
            // offers what it would have done to a fighter and the creature
            // takes what it can. Nothing at all, unless it is susceptible.
            // A break is its own window: the control a guard breaker would
            // have offered is not taken on top of it.
            if guarded == monster::Guarded::Lands {
                beast.take_control(monster::Control {
                    launch: m.launch,
                    grabs: m.grabs,
                    ..monster::Control::default()
                });
            }
            self.players[i].heal(m.leeched(dealt));
            self.players[i].hit_used = true;
            if dealt > 0 {
                dual::landed_a_hit(&mut self.players[i]);
                spell_lands_on_creature(&mut beast, &mut self.players, i, kind);
            }
            // **No freeze here, on either side.** The animal does not stop for
            // one fighter, and a fighter who stopped for it was held still
            // inside the reach of something that had not: the scripted hunter
            // lost every one of eighteen hunts the day this froze the attacker.
            // The weight of a blow on the creature is its flinch and its
            // poise to show, not a pause in the fighter.
        }

        // Nothing to stand on any more -- for whoever was standing on this one.
        if !beast.alive() {
            let now = herd(&beast, &self.monsters);
            for p in self.players.iter_mut() {
                if p.aboard() && mount_slot(p.mount) == slot {
                    fall_off(p, &now);
                }
            }
        }
        self.monsters[slot] = Some(beast);
        // What its moves do besides hurt, with the whole world to do it in.
        if let Some(hook) = beast.sp().fight.landed {
            for (i, hit) in struck.into_iter().enumerate() {
                if let Some((kind, guarded)) = hit {
                    hook(self, slot, i, kind, guarded);
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// The hunt's shared machinery: senses, the floor, falls, defended things
// (bestiary P4 to P7)
// ---------------------------------------------------------------------------

/// The most fires a frame can hold at once: every effect, every stone, every
/// bolt.
const MAX_FIRES: usize = MAX_EFFECTS + crate::stones::MAX_STONES + MAX_BOLTS;

impl World {
    /// **What each creature's senses allow its glance this frame** (P5): its
    /// species' perception filter asked of every fighter, and -- for a species
    /// that hears -- the noise it would attend to.
    ///
    /// A species without the senses row is the Ridgeback's glance exactly:
    /// it perceives everybody, and nothing is asked, so a fight without
    /// filters costs nothing here and hashes as it did.
    fn senses(&self, ground: &Terrain, field: &Field) -> [Senses; MAX_MONSTERS] {
        let mut out = [Senses::ALL; MAX_MONSTERS];
        if !self.monsters.iter().flatten().any(|m| m.sp().fight.row) {
            return out;
        }
        let seen = self.players;
        let effects = self.effects;
        let herd = self.monsters;
        let crowd = self.critters;
        let scene = Scene {
            stones: field,
            players: &seen,
            effects: &effects,
            quarry: &herd,
            critters: &crowd,
            arena: ground,
        };
        for (slot, beast) in self.monsters.iter().enumerate() {
            let Some(m) = beast else { continue };
            let sp = m.sp();
            if !sp.fight.row {
                continue;
            }
            let head = m.head();
            for (i, p) in self.players.iter().enumerate() {
                let q = Quarry {
                    pos: p.pos,
                    vel: p.vel,
                    alive: p.health > 0,
                    aboard: p.aboard(),
                    stunned: p.action.stunned(),
                };
                out[slot].perceived[i] = (sp.fight.perceives)(&crate::perception::Perceiver {
                    monster: m,
                    slot,
                    head,
                    quarry: &q,
                    who: i,
                    scene: &scene,
                    lore: &self.lore,
                });
            }
            if sp.fight.hears {
                out[slot].heard = self.heard_by(m, head);
            }
        }
        out
    }

    /// **The noise a creature attends to**: of those made since its last
    /// glance, the one that reached its head with the most to spare -- its
    /// loudness to this species, less its distance. `None` if nothing reached.
    pub fn heard_by(&self, m: &Monster, head: V3) -> Option<monster::Heard> {
        let sp = m.sp();
        let window = m.glance_frames().max(1) as u32;
        // Hunger, or deafness: its species' say on how far it hears now.
        let hearing = sp.fight.hearing.map(|f| f(m, &self.lore));
        let mut best: Option<(Fx, monster::Heard)> = None;
        for n in crate::noise::all(&self.lore) {
            if self.frame.wrapping_sub(n.born) > window {
                continue;
            }
            let loud = match hearing {
                Some(mul) => crate::noise::loudness(sp, &n).mul(mul),
                None => crate::noise::loudness(sp, &n),
            };
            let spare = loud.sub(crate::math::wide_len(n.pos().sub(head)));
            if spare.raw() <= 0 || best.is_some_and(|(b, _)| b.raw() >= spare.raw()) {
                continue;
            }
            best = Some((
                spare,
                monster::Heard {
                    at: n.pos(),
                    who: n.who,
                },
            ));
        }
        best.map(|(_, h)| h)
    }

    /// **What the floor does to whoever is standing in it** (P4): the walk
    /// slowed through the ordinary slow, damage ticking, a pull toward the
    /// middle, a lift up the column -- for the fighters, the creatures and the
    /// small bodies its kind reaches. The dodge and the jump read the same
    /// floor where they are thrown (`step_player`).
    fn feel_the_floor(&mut self, ground: &Terrain) {
        let floor = &ground.floor;
        if floor.is_empty() {
            return;
        }
        let frame = self.frame;
        for p in self.players.iter_mut() {
            if p.health <= 0 {
                continue;
            }
            let u = floor.underfoot(p.pos, hazard::reach::FIGHTERS, p.footed(), frame);
            if u.walk.raw() < Fx::ONE.raw() {
                // Two frames, so the slow is still on when the step that
                // follows reads it, and lifts the frame after the feet leave.
                p.slow(2, u.walk);
            }
            if u.damage > 0 {
                p.wound(u.damage);
            }
            let (to, speed) = u.pull;
            if speed.raw() > 0 && !p.aboard() {
                let gap = V3::new(to.x.sub(p.pos.x), Fx::ZERO, to.z.sub(p.pos.z));
                let far = crate::math::wide_flat_len(gap);
                let step = speed.mul(crate::DT).min(far);
                p.pos = p.pos.add(crate::math::wide_normalized(gap).scale(step));
            }
            // **Only a lift lifts.** With no lift underfoot this is zero, and
            // zero is above every falling body's speed: the first version set
            // every falling fighter's fall to nothing, every frame, in any
            // fight with a hazard on the floor -- a hook's launch came down at
            // walking pace (found by the Hornback, whose boulders are hazards).
            if u.lift.raw() > 0 && u.lift.raw() > p.vel.y.raw() && !p.aboard() {
                p.vel.y = u.lift;
                p.grounded = false;
            }
        }
        for m in self.monsters.iter_mut().flatten() {
            if !m.alive() {
                continue;
            }
            let u = floor.underfoot(m.pos, hazard::reach::CREATURES, true, frame);
            m.scorch(u.damage);
            if u.walk.raw() < Fx::ONE.raw() {
                m.slowed = m.slowed.max(2);
                m.slow_mul = m.slow_mul.min(u.walk);
            }
        }
        if let Some(sp) = floor.species() {
            let _ = sp;
            for i in 0..crate::critter::MAX_CRITTERS {
                let c = self.critters[i];
                if !c.alive() {
                    continue;
                }
                let u = floor.underfoot(c.pos, hazard::reach::CRITTERS, true, frame);
                if u.damage > 0 {
                    pack::hurt(
                        &mut self.pack,
                        &mut self.critters,
                        i,
                        u.damage,
                        V3::ZERO,
                        Fx::ZERO,
                        Fx::ZERO,
                    );
                }
            }
        }
    }

    /// **Falls** (P6): the rule for each fighter, against the body the frame
    /// started with. See [`fall_rule`].
    fn land(&mut self, before: &[Player; MAX_PLAYERS]) {
        let lowest = self.terrain().lowest();
        for (was, p) in before.iter().zip(self.players.iter_mut()) {
            let damage = fall_rule_over(was, p, lowest);
            if damage > 0 && p.health > 0 {
                p.wound(damage);
            }
        }
    }

    /// **Every fire in the world this frame**: the effects that are fire
    /// (`EffectKind::ignites`), lit stones, and fire bolts in flight. What puts
    /// fire to the floor hazards.
    pub fn fires(&self) -> ([hazard::Fire; MAX_FIRES], usize) {
        let mut out = [hazard::Fire {
            at: V3::ZERO,
            radius: Fx::ZERO,
        }; MAX_FIRES];
        let mut n = 0;
        let mut add = |at: V3, radius: Fx| {
            if n < MAX_FIRES {
                out[n] = hazard::Fire { at, radius };
                n += 1;
            }
        };
        for e in self.effects.iter().flatten() {
            if !e.kind.ignites() {
                continue;
            }
            match e.kind {
                EffectKind::FireTornado => add(e.tornado_pos(), e.field_radius()),
                EffectKind::FireRing => add(e.pos, e.ring_radius()),
                _ => add(e.pos, e.field_radius()),
            }
        }
        for stone in stones::gather(&self.players).iter().flatten() {
            if stone.lit > 0 {
                add(stone.at, stone.radius());
            }
        }
        for b in self.bolts.iter().flatten() {
            add(b.pos, Fx::ZERO);
        }
        (out, n)
    }

    /// **The hunt's lore, a frame on**: fire put to the floor and spreading
    /// along it, every hazard aging and becoming what its kind becomes, and
    /// the defended things rolling on while they are escorted.
    fn step_the_lore(&mut self) {
        if self.lore.owner.is_none() {
            return;
        }
        let (fires, n) = self.fires();
        hazard::step(&mut self.lore, &self.monsters, &fires[..n]);
        let hunters: [V3; MAX_PLAYERS] = std::array::from_fn(|i| self.players[i].pos);
        let alive: [bool; MAX_PLAYERS] = std::array::from_fn(|i| self.players[i].health > 0);
        let mut near = [V3::ZERO; MAX_PLAYERS];
        let mut count = 0;
        for i in 0..MAX_PLAYERS {
            if alive[i] {
                near[count] = hunters[i];
                count += 1;
            }
        }
        objective::step(&mut self.lore, self.arena.get(), &near[..count]);
    }

    /// **A creature's blow on the defended things** (P7): its move's volume,
    /// the one it reaches a fighter with, against each one's box -- once per
    /// move, for the move's damage times what the thing takes of it.
    fn strike_the_objectives(&mut self, slot: usize, beast: &Monster) {
        let bit = 1u8 << slot.min(7);
        let mut hits = [(0usize, 0i32); objective::MAX_STANDING];
        let mut n = 0;
        let lore = self.lore;
        for o in objective::standing(&lore, self.arena.get()) {
            let mut state = o.state;
            let Doing::Active { kind, .. } = beast.doing else {
                if state.struck & bit != 0 {
                    state.struck &= !bit;
                    objective::set(&mut self.lore, o.index, state);
                }
                continue;
            };
            if state.broken || state.struck & bit != 0 {
                continue;
            }
            let height = o.site.extent().y;
            if beast.reaches(o.at, height, o.radius()) && n < hits.len() {
                hits[n] = (o.index, beast.sp().attack(kind).damage);
                n += 1;
            }
        }
        for (i, damage) in hits.into_iter().take(n) {
            let mut state = objective::get(&self.lore, i);
            state.struck |= bit;
            objective::set(&mut self.lore, i, state);
            objective::strike(&mut self.lore, i, damage);
        }
    }

    /// **The critters' blows on the defended things** (P7): each critter's
    /// move volume against each one's box, once a move, as a creature's.
    fn critters_strike_the_objectives(&mut self) {
        let Some(pack) = self.pack else { return };
        let sp = pack.sp();
        let lore = self.lore;
        for o in objective::standing(&lore, self.arena.get()) {
            if o.state.broken {
                continue;
            }
            let mut damage = 0;
            let bit = 1u8 << o.index;
            for c in 0..critter::MAX_CRITTERS {
                let body = &mut self.critters[c];
                if !body.alive() || body.state != critter::is::ACTIVE {
                    body.struck_objectives &= !bit;
                    continue;
                }
                if body.struck_objectives & bit != 0 {
                    continue;
                }
                if body.reaches(sp, o.at, o.site.extent().y, o.radius()) {
                    body.struck_objectives |= bit;
                    damage += sp.attack(body.act).damage;
                }
            }
            if damage > 0 {
                objective::strike(&mut self.lore, o.index, damage);
            }
        }
    }

    /// **The fight's own floor signs** this frame: its species'
    /// `FightDecl::signs`, or none. What the renderer draws besides the
    /// bodies' telegraphs (`crate::sign`).
    pub fn signs(&self) -> crate::sign::Signs {
        let mut out = crate::sign::Signs::NONE;
        if let Some(f) = self.lore.owner.and_then(|s| s.get().fight.signs) {
            f(self, &mut out);
        }
        out
    }

    /// **What the fight's species draws besides its hazards and telegraphs**
    /// (`FightDecl::marks`): read by the renderer and the overlay, and by the
    /// tests that pin the drawing to the rule.
    pub fn marks(&self) -> crate::species::Marks {
        let mut out = crate::species::Marks::NONE;
        if let Some(hook) = self.lore.owner.and_then(|s| s.get().fight.marks) {
            hook(self, &mut out);
        }
        out
    }

    /// **A body drawn that is not in the world** (`FightDecl::apparition`),
    /// and the strength it is drawn at: the Veilstalker's mimic. The
    /// renderer draws it as it draws a creature; nothing else reads it.
    pub fn apparition(&self) -> Option<(Monster, Fx)> {
        let hook = self.lore.owner.and_then(|s| s.get().fight.apparition)?;
        hook(self)
    }

    /// How much of a creature part can be seen, nought to one: its species'
    /// `shown`, or fully. What the renderer draws a part at, and what the
    /// report and the scripted hunter call visible.
    pub fn shown(&self, slot: usize, part: usize) -> Fx {
        let Some(m) = self.monsters.get(slot).and_then(|m| m.as_ref()) else {
            return Fx::ZERO;
        };
        match m.sp().fight.shown {
            Some(f) => f(self, slot, part).clamp(Fx::ZERO, Fx::ONE),
            None => Fx::ONE,
        }
    }
}

#[cfg(test)]
mod fury_tests {
    //! The Champion's rework (exploration 0015) at the level of one blow: the
    //! things a fight is too noisy to pin.
    use super::*;

    fn rushing_champion(left: u16) -> Player {
        let mut p = Player::new(Class::Champion);
        if let Mechanic::Forms { rush, rush_vel, .. } = &mut p.mechanic {
            *rush = left;
            *rush_vel = V3::new(t::rush_speed(), Fx::ZERO, Fx::ZERO);
        }
        p
    }

    fn blow(damage: i32) -> Hit {
        Hit {
            damage,
            hitstun: 20,
            blockstun: 10,
            knockback: Fx::ONE,
            launch: Fx::ZERO,
            grabs: 0,
            by: 1,
            dir: V3::new(Fx::ONE, Fx::ZERO, Fx::ZERO),
            blocked: false,
            parried: false,
            interrupts: true,
        }
    }

    #[test]
    fn a_rush_takes_a_share_and_keeps_going() {
        let mut p = rushing_champion(t::rush_eat_frames() + 5);
        let before = p.health;
        apply_hit(&mut p, blow(60));
        assert_eq!(
            before - p.health,
            60 * t::rush_damage_taken() as i32 / 100,
            "a braced blow costs its share"
        );
        assert!(
            p.action.actionable(),
            "and does not stop him: {:?}",
            p.action
        );
        assert!(rushing(&p).is_some(), "the run goes on");
    }

    #[test]
    fn the_last_frames_eat_one_blow_whole_and_only_one() {
        let mut p = rushing_champion(t::rush_eat_frames());
        let before = p.health;
        apply_hit(&mut p, blow(80));
        assert_eq!(p.health, before, "the shoulder down eats it whole");
        apply_hit(&mut p, blow(80));
        assert!(p.health < before, "but only once a Rush");
    }

    #[test]
    fn enough_damage_breaks_the_rush() {
        let mut p = rushing_champion(t::rush_eat_frames() + 5);
        let big = (t::rush_breaks() + 1) * 100 / t::rush_damage_taken().max(1) as i32 + 1;
        apply_hit(&mut p, blow(big));
        assert!(
            matches!(p.action, Action::HitStun { .. }),
            "a bite past the break stops him the ordinary way: {:?}",
            p.action
        );
        assert!(rushing(&p).is_none(), "and ends the run");
    }

    #[test]
    fn a_creatures_blow_meeting_a_heavy_one_is_spent_on_it() {
        let mut p = Player::new(Class::Champion);
        p.action = Action::Active {
            kind: moves::champion::METEOR,
            left: 2,
        };
        let before = p.health;
        let mut hit = blow(300);
        hit.by = QUARRY;
        apply_hit(&mut p, hit);
        assert_eq!(p.health, before, "the bite took nothing");
        assert!(
            matches!(p.mechanic, Mechanic::Forms { fury, .. } if fury.clash_by == QUARRY),
            "and who threw it is remembered for the settling"
        );
        let mut q = Player::new(Class::Champion);
        let mut world = blow(30);
        world.by = NOBODY;
        q.action = p.action;
        let before = q.health;
        apply_hit(&mut q, world);
        assert!(q.health < before, "the world's own blows are not clashed");
    }

    #[test]
    fn only_heavy_blows_and_a_shoulder_down_clash() {
        let mut p = Player::new(Class::Champion);
        p.action = Action::Active {
            kind: moves::champion::SHOCKWAVE,
            left: 2,
        };
        assert!(clashing(&p), "a full charge's active frames clash");
        p.action = Action::Active {
            kind: moves::champion::EARTHBREAKER,
            left: 2,
        };
        assert!(clashing(&p), "so does the hammer's third link");
        p.action = Action::Active {
            kind: moves::champion::SWORD_GROUND,
            left: 2,
        };
        assert!(!clashing(&p), "a sword cut does not");
        let mut q = rushing_champion(t::rush_eat_frames());
        assert!(clashing(&q), "a Rush with the shoulder down does");
        if let Mechanic::Forms { fury, .. } = &mut q.mechanic {
            fury.eaten = true;
        }
        assert!(!clashing(&q), "once a run");
        let mut r = Player::new(Class::Bulwark);
        r.action = Action::Active { kind: 1, left: 2 };
        assert!(!clashing(&r), "and nobody else's does");
    }

    #[test]
    fn a_landed_third_hit_gives_the_rush_back() {
        let mut p = Player::new(Class::Champion);
        if let Mechanic::Forms { recharge, .. } = &mut p.mechanic {
            *recharge = 50;
        }
        p.action = Action::Active {
            kind: moves::champion::UPCUT,
            left: 2,
        };
        confirm_chain(&mut p, false);
        assert!(
            matches!(p.mechanic, Mechanic::Forms { recharge: 0, .. }),
            "the third hit refunds the Rush"
        );
        let mut q = Player::new(Class::Champion);
        if let Mechanic::Forms { recharge, .. } = &mut q.mechanic {
            *recharge = 50;
        }
        q.action = Action::Active {
            kind: moves::champion::BACKCUT,
            left: 2,
        };
        confirm_chain(&mut q, false);
        assert!(
            matches!(q.mechanic, Mechanic::Forms { recharge: 50, .. }),
            "a second hit does not"
        );
    }
}

#[cfg(test)]
mod knockback_tests {
    //! The knockback rules between fighters (`docs/design/stun.md`) at the
    //! level of one blow. What a real fight does with them is
    //! `tests/knockback.rs`.
    use super::*;

    /// A shove and a launch to measure by: real speeds from the Oven, so
    /// nothing here is a number of its own.
    fn shove() -> Fx {
        t::move_speed()
    }

    fn lift() -> Fx {
        t::move_speed()
    }

    fn blow(by: u8) -> Hit {
        Hit {
            damage: 50,
            hitstun: 20,
            blockstun: 10,
            knockback: shove(),
            launch: Fx::ZERO,
            grabs: 0,
            by,
            dir: V3::new(Fx::ONE, Fx::ZERO, Fx::ZERO),
            blocked: false,
            parried: false,
            interrupts: true,
        }
    }

    fn hurt_to(class: Class, share: (i32, i32)) -> Player {
        let mut p = Player::new(class);
        p.health = p.full_health() * share.0 / share.1;
        p
    }

    #[test]
    fn a_fighters_blow_swells_with_what_the_victim_has_lost() {
        let mut fresh = Player::new(Class::Champion);
        let mut hurt = hurt_to(Class::Champion, (1, 2));
        apply_hit(&mut fresh, blow(1));
        apply_hit(&mut hurt, blow(1));
        assert!(
            hurt.vel.x.raw() > fresh.vel.x.raw(),
            "half a bar down is thrown no further: {:?} against {:?}",
            hurt.vel.x,
            fresh.vel.x
        );
        assert!(
            hurt.stun_total > fresh.stun_total,
            "nor held any longer: {} against {}",
            hurt.stun_total,
            fresh.stun_total
        );
        assert!(
            fresh.stun_total >= 20,
            "never less than the table's own stun"
        );
    }

    #[test]
    fn the_shove_starts_above_the_table_and_ends_where_it_did() {
        // A Champion weighs one, so what is measured is the swell alone.
        let mut fresh = Player::new(Class::Champion);
        let mut nothing = blow(1);
        nothing.damage = 0;
        apply_hit(&mut fresh, nothing);
        assert_eq!(
            fresh.vel.x,
            shove().mul(t::swell_start()),
            "untouched, the shove is the start times the table"
        );
        assert!(
            t::swell_start().raw() > Fx::ONE.raw(),
            "and the start is above one"
        );
        // The killing blow: the ceiling, which the start does not move.
        let mut last = Player::new(Class::Champion);
        let finishing = blow(1);
        last.health = finishing.damage;
        apply_hit(&mut last, finishing);
        let share = Fx::ratio(finishing.damage, last.full_health());
        let ceiling = Fx::ONE
            .add(t::swell_knockback())
            .add(t::swell_blow().mul(share));
        assert!(
            (last.vel.x.raw() - shove().mul(ceiling).raw()).abs() < 64,
            "at death the ceiling: {:?} against {:?}",
            last.vel.x,
            shove().mul(ceiling)
        );
    }

    #[test]
    fn a_launch_starts_at_the_table() {
        let mut p = Player::new(Class::Champion);
        let mut up = blow(1);
        up.damage = 0;
        up.launch = lift();
        apply_hit(&mut p, up);
        assert_eq!(p.vel.y, lift(), "the follow-ups are tuned to it");
    }

    #[test]
    fn a_creatures_blow_does_what_its_table_says() {
        for by in [QUARRY, NOBODY] {
            let mut p = hurt_to(Class::DualMage, (1, 4));
            apply_hit(&mut p, blow(by));
            assert_eq!(p.vel.x, shove(), "no swell and no weight");
            assert_eq!(p.stun_total, 20, "and the stun as written");
            assert_eq!(p.taken, 0, "and it starts no string");
        }
    }

    #[test]
    fn a_heavy_travels_less_than_a_light_from_the_same_blow() {
        let mut heavy = Player::new(Class::Bulwark);
        let mut light = Player::new(Class::DualMage);
        apply_hit(&mut heavy, blow(0));
        apply_hit(&mut light, blow(0));
        assert!(heavy.vel.x.raw() < light.vel.x.raw());
        assert_eq!(
            heavy.stun_total, light.stun_total,
            "weight divides knockback, never stun"
        );
        let (mut heavy, mut light) = (Player::new(Class::Bulwark), Player::new(Class::DualMage));
        let mut guarded = blow(0);
        guarded.blocked = true;
        apply_hit(&mut heavy, guarded);
        apply_hit(&mut light, guarded);
        assert!(
            heavy.vel.x.raw() < light.vel.x.raw(),
            "and blocked pushback with it"
        );
    }

    #[test]
    fn each_hit_in_a_string_keeps_less_of_the_swell() {
        let mut p = hurt_to(Class::Champion, (1, 2));
        let mut stuns = Vec::new();
        for _ in 0..4 {
            p.health = p.full_health() / 2;
            apply_hit(&mut p, blow(0));
            stuns.push(p.stun_total);
        }
        assert!(
            stuns.windows(2).all(|w| w[1] <= w[0]) && stuns[3] < stuns[0],
            "a string does not fade: {stuns:?}"
        );
        assert!(stuns.iter().all(|s| *s >= 20), "below the table: {stuns:?}");
        // Free again: the string is over.
        p.action = Action::HitStun { left: 0 };
        countdown(&mut p, false, Input::default());
        assert_eq!(p.taken, 0);
    }

    #[test]
    fn a_grab_keeps_its_launch_for_the_release() {
        let mut p = Player::new(Class::Champion);
        let mut grab = blow(1);
        grab.grabs = 10;
        grab.launch = lift();
        apply_hit(&mut p, grab);
        assert!(matches!(p.action, Action::Held { .. }));
        assert!(p.thrown.raw() > 0, "the launch was dropped");
        let banked = p.thrown;
        p.action = Action::Held { left: 0 };
        countdown(&mut p, false, Input::default());
        assert_eq!(p.vel.y, banked, "let go, and thrown");
        assert!(!p.grounded);
        assert_eq!(p.thrown, Fx::ZERO);
    }

    #[test]
    fn steering_a_launch_turns_it_and_never_shortens_it() {
        let mut p = Player::new(Class::Champion);
        p.vel = V3::new(shove(), Fx::ZERO, Fx::ZERO);
        let before = V3::new(p.vel.x, Fx::ZERO, p.vel.z).flat_len();
        let air = p.class.mobility().air_speed;
        for _ in 0..10 {
            steer_while_stunned(&mut p, V3::new(Fx::ZERO, Fx::ZERO, Fx::ONE), air);
        }
        let after = V3::new(p.vel.x, Fx::ZERO, p.vel.z).flat_len();
        assert!(p.vel.z.raw() > 0, "holding sideways did not turn it");
        assert!(
            after.raw() >= before.raw() - 64,
            "steering cut the launch: {before:?} -> {after:?}"
        );
        // Holding back against it: it may turn, it may not brake.
        let mut q = Player::new(Class::Champion);
        q.vel = V3::new(shove(), Fx::ZERO, Fx::ZERO);
        steer_while_stunned(&mut q, V3::new(Fx::ONE.neg(), Fx::ZERO, Fx::ZERO), air);
        assert!(V3::new(q.vel.x, Fx::ZERO, q.vel.z).flat_len().raw() >= before.raw() - 64);
    }
}
