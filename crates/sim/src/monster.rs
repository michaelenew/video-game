//! A creature you fight, and stand on.
//!
//! See `docs/design/monsters.md` for what the fight is and why, told through
//! the Ridgeback, the first creature this was built for. Everything here is
//! **species-generic**: what one kind of creature is -- its skeleton, its
//! parts, its legs, its moves, its clips and its numbers -- is a table in
//! [`crate::species`], and a `Monster` carries the id of its table in the
//! snapshot and reads the rest from there.
//!
//! Four pieces live here. The fifth -- the skeleton itself -- is
//! [`crate::beast`].
//!
//! **Moves**, read live from the Oven, each with its own frame data, hit
//! volume, the bearing and range it wants its target on, and a lockout so that
//! a move you have just baited cannot come straight back.
//!
//! **A pose that is a lookup.** Every frame, what the creature looks like is
//! sampled out of a baked clip table by *phase*, layered with a gait indexed by
//! ground covered and whatever its broken legs are doing to it. All of it a
//! pure function of the snapshot, which is what keeps rollback exact.
//!
//! **A ladder of ways to be in trouble.** A flinch is cheap and short. A
//! stumble is a knee on the ground -- what a broken leg and a landed piece of
//! crowd control both produce, and the only way onto its back from the floor.
//! A topple is the long one the climb is for.
//!
//! **A control algorithm that glances rather than watching.** It samples the
//! player every few frames and extrapolates from that sample until the next
//! one, so it cannot read an input and a change of direction between glances is
//! a real thing to get good at.

use crate::DT;
pub use crate::beast::{Rig, Shape};

use crate::arena::{Bounds, Terrain};
use crate::beast::{self, MAX_BREAKABLE, Pose};
use crate::fixed::Fx;
use crate::math::V3;
pub use crate::species::{MAX_MOVES, Species, SpeciesId};
use crate::tuning as t;

/// The smallest positive value 16.16 can hold. A guard against dividing by a
/// zero, not a magnitude anybody picked.
const SMALLEST: Fx = Fx::from_raw(1);

/// Not standing on anything. Shares the sentinel with `state::NOBODY` on
/// purpose: both mean "this index refers to nothing".
pub const NO_PART: u8 = u8::MAX;

/// How many creatures a hunt can hold at once. Two: the Pair are two full
/// monsters, and every other creature is one. See `state::World::monsters`.
pub const MAX_MONSTERS: usize = 2;

/// Every creature slot a world has. `None` is an empty slot; a versus match is
/// all of them empty.
pub type Herd = [Option<Monster>; MAX_MONSTERS];

/// Which creature, and which of its parts, a rider is standing on -- packed
/// into the one byte a fighter keeps for it. The slot in the top two bits,
/// the part in the low six ([`beast::MAX_PARTS`] fits). [`NO_PART`] is nobody.
pub const fn mount_of(slot: usize, part: usize) -> u8 {
    ((slot << 6) | (part & 0x3F)) as u8
}

/// The creature slot a packed mount names.
pub const fn mount_slot(mount: u8) -> usize {
    (mount >> 6) as usize
}

/// The part a packed mount names.
pub const fn mount_part(mount: u8) -> usize {
    (mount & 0x3F) as usize
}

// ---------------------------------------------------------------------------
// Moves
// ---------------------------------------------------------------------------

/// One of the creature's moves, read live from the Oven.
///
/// The hit volume is a vertical cylinder **in body space**: an anchor in the
/// creature's own coordinates, a radius, and a height span. Body space rather
/// than world space because that is where the move is authored -- "the head,
/// one metre in front of it" does not change when the creature turns -- and
/// because it makes `follows` free: the anchor rides the same bone the part
/// does.
#[derive(Clone, Copy, Debug)]
pub struct Attack {
    pub name: &'static str,
    pub startup: u16,
    pub active: u16,
    pub recovery: u16,
    pub damage: i32,
    pub hit_x: Fx,
    pub hit_z: Fx,
    pub hit_radius: Fx,
    pub hit_low: Fx,
    pub hit_high: Fx,
    /// The bone the volume rides: the head's, the tail tip's, or the body's.
    pub follows: u8,
    pub hitstun: u16,
    pub blockstun: u16,
    pub knockback: Fx,
    pub launch: Fx,
    pub unblockable: bool,
    /// Forward speed while the move is active. The charge is a move that
    /// travels; the rest are moves that happen where you are standing.
    pub advance: Fx,
    // ---- what the control algorithm reads ----
    /// The distance the move is for.
    pub ideal_range: Fx,
    /// How far either side of that it is still worth throwing. Outside this the
    /// move scores nothing at all, which is what stops it being thrown at a
    /// target it cannot reach.
    pub range_span: Fx,
    /// Cosine of the bearing the move wants its target on: one is dead ahead,
    /// zero is beside, minus one is behind. The tail sweep's is negative, which
    /// is the whole of how it knows what it is for.
    pub aim_cos: Fx,
    /// How far either side of that bearing it is still worth throwing. Wide for
    /// a sweep, narrow for a charge, irrelevant for the shake.
    pub aim_span: Fx,
    /// Base appetite for the move, before anything about the situation.
    pub weight: i32,
    /// Added per player standing on the creature. This is what makes a rider
    /// the only thing it cares about.
    pub rider_weight: i32,
    /// Frames before it can throw this move again, counted from the frame it
    /// started.
    ///
    /// **This is what makes a bait worth making.** Without it, the answer to
    /// every buck is another buck: a rider who reads the shake, leaves the
    /// ground over it and lands back on the animal has earned nothing, because
    /// the next shake starts on the frame the last one ended. With it, reading
    /// a move buys a window whose length you can learn. See
    /// `docs/design/monsters.md` §3.
    pub cooldown: u16,
    /// How fast the hit volume travels out along the facing while the move is
    /// active. Zero for a move that happens where the animal is standing.
    pub travel: Fx,
    /// Frames the fighter it hits is rooted to the spot. Zero is an ordinary
    /// hit; the spray's is what the charge after it is for.
    pub root: u16,
}

impl Attack {
    pub const fn total(&self) -> u16 {
        self.startup + self.active + self.recovery
    }

    /// Frames the creature is still busy after the first active frame lands.
    pub const fn busy_after_contact(&self) -> i32 {
        (self.active as i32 - 1) + self.recovery as i32
    }

    /// How long the creature is helpless once the move is over its hit. This is
    /// the window the fight is built around, and the fight report measures it.
    pub const fn opening(&self) -> u16 {
        self.recovery
    }
}

/// A move's hit, as it will be drawn: where it lands, how wide and how tall,
/// and how far it goes from there. See [`Monster::telegraph`].
#[derive(Clone, Copy, Debug)]
pub struct Telegraph {
    pub kind: u8,
    /// Where the volume is on the first frame it is out -- or now, if it is.
    pub anchor: V3,
    pub radius: Fx,
    pub low: Fx,
    pub high: Fx,
    /// Which way it goes, if it goes anywhere: the animal's facing.
    pub along: V3,
    /// How far the volume travels from `anchor` over the frames it has left.
    /// Zero for everything that lands where it is thrown; the charge and the
    /// spray have a lane.
    pub sweep: Fx,
    /// Through the windup, from nought to one. One while the hit is out.
    pub progress: Fx,
    /// The hit is out.
    pub live: bool,
}

impl Species {
    /// Read a move from the live tuning store.
    pub fn attack(&self, kind: u8) -> Attack {
        use crate::oven::{MonsterField as F, species_raw};
        let slot = (kind as usize).min(self.moves.len() - 1);
        let raw = |f: F| species_raw(self.id, self.move_index(slot, f));
        Attack {
            name: self.moves[slot].name,
            startup: raw(F::Startup) as u16,
            active: raw(F::Active) as u16,
            recovery: raw(F::Recovery) as u16,
            damage: raw(F::Damage),
            hit_x: Fx::from_raw(raw(F::HitX)),
            hit_z: Fx::from_raw(raw(F::HitZ)),
            hit_radius: Fx::from_raw(raw(F::HitRadius)),
            hit_low: Fx::from_raw(raw(F::HitLow)),
            hit_high: Fx::from_raw(raw(F::HitHigh)),
            follows: raw(F::Follows) as u8,
            hitstun: raw(F::Hitstun) as u16,
            blockstun: raw(F::Blockstun) as u16,
            knockback: Fx::from_raw(raw(F::Knockback)),
            launch: Fx::from_raw(raw(F::Launch)),
            unblockable: raw(F::Unblockable) != 0,
            advance: Fx::from_raw(raw(F::Advance)),
            ideal_range: Fx::from_raw(raw(F::IdealRange)),
            range_span: Fx::from_raw(raw(F::RangeSpan)),
            aim_cos: Fx::from_raw(raw(F::AimCos)),
            aim_span: Fx::from_raw(raw(F::AimSpan)),
            weight: raw(F::Weight),
            rider_weight: raw(F::RiderWeight),
            cooldown: raw(F::Cooldown) as u16,
            travel: Fx::from_raw(raw(F::Travel)),
            root: raw(F::Root) as u16,
        }
    }

    /// Every move, live, in the move table's order.
    pub fn attacks(&self) -> impl Iterator<Item = Attack> + '_ {
        (0..self.moves.len()).map(|kind| self.attack(kind as u8))
    }
}

// ---------------------------------------------------------------------------
// What it is doing
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Doing {
    /// Standing, walking, turning. The only state a move can start from.
    Prowl,
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
    /// A flinch. Short, and it interrupts whatever was happening.
    Flinch {
        left: u16,
    },
    /// Down on a knee.
    ///
    /// The middle rung of the ladder, and the one the ground game is aimed at:
    /// a broken foot puts the creature here, and so does a piece of crowd
    /// control that lands while it is susceptible. The front or back end drops
    /// far enough to climb, which is what makes breaking a leg a route up
    /// rather than a slightly faster kill.
    Stumble {
        left: u16,
        /// True when it is the front end that has gone down.
        front: bool,
    },
    /// Poise broken. The long window the climb is for.
    Toppled {
        left: u16,
    },
    Dead,
}

impl Doing {
    pub const fn attacking(self) -> Option<u8> {
        match self {
            Doing::Startup { kind, .. }
            | Doing::Active { kind, .. }
            | Doing::Recovery { kind, .. } => Some(kind),
            _ => None,
        }
    }

    /// Can a new move start?
    pub const fn free(self) -> bool {
        matches!(self, Doing::Prowl)
    }

    /// Is the creature currently punishable -- out of its hit and unable to
    /// answer? What the fight report counts as an opening.
    pub const fn open(self) -> bool {
        matches!(
            self,
            Doing::Recovery { .. }
                | Doing::Flinch { .. }
                | Doing::Stumble { .. }
                | Doing::Toppled { .. }
        )
    }

    /// Is it down far enough to climb onto from the floor?
    pub const fn grounded_route(self) -> bool {
        matches!(self, Doing::Stumble { .. } | Doing::Toppled { .. })
    }

    pub const fn tag(self) -> u32 {
        match self {
            Doing::Prowl => 0,
            Doing::Startup { .. } => 1,
            Doing::Active { .. } => 2,
            Doing::Recovery { .. } => 3,
            Doing::Flinch { .. } => 4,
            Doing::Toppled { .. } => 5,
            Doing::Dead => 6,
            Doing::Stumble { .. } => 7,
        }
    }

    /// Which clip this state is drawn from. Two states with the same key are
    /// two points on one continuous motion; two with different keys are a
    /// cut, and a cut is not a movement of the surface under a rider's feet.
    ///
    /// `moves` is how many moves the species has: the states that are not a
    /// move are numbered after them.
    pub const fn clip_key(self, moves: usize) -> u8 {
        let moves = moves as u8;
        match self {
            Doing::Startup { kind, .. }
            | Doing::Active { kind, .. }
            | Doing::Recovery { kind, .. } => kind,
            Doing::Prowl => moves,
            Doing::Flinch { .. } => moves + 1,
            Doing::Stumble { .. } => moves + 2,
            Doing::Toppled { .. } => moves + 3,
            Doing::Dead => moves + 4,
        }
    }

    pub const fn frames_left(self) -> u16 {
        match self {
            Doing::Prowl | Doing::Dead => 0,
            Doing::Startup { left, .. }
            | Doing::Active { left, .. }
            | Doing::Recovery { left, .. }
            | Doing::Flinch { left }
            | Doing::Stumble { left, .. }
            | Doing::Toppled { left } => left,
        }
    }
}

/// Which phase, and how far through it, as a fraction.
fn phase(species: &Species, doing: Doing) -> (u8, Fx) {
    let Some(kind) = doing.attacking() else {
        return (3, Fx::ZERO);
    };
    let m = species.attack(kind);
    let part = |left: u16, span: u16| {
        if span == 0 {
            return Fx::ONE;
        }
        Fx::from_int(span.saturating_sub(left) as i32).div(Fx::from_int(span as i32))
    };
    match doing {
        Doing::Startup { left, .. } => (0, part(left, m.startup)),
        Doing::Active { left, .. } => (1, part(left, m.active)),
        Doing::Recovery { left, .. } => (2, part(left, m.recovery)),
        _ => (3, Fx::ZERO),
    }
}

/// How far through a countdown we are, as a fraction: zero at the start.
fn through(left: u16, total: u16) -> Fx {
    let total = total.max(1);
    Fx::from_int(total.saturating_sub(left.min(total)) as i32).div(Fx::from_int(total as i32))
}

// ---------------------------------------------------------------------------
// Crowd control
// ---------------------------------------------------------------------------

/// What a fighter's move would do to another fighter, offered to the creature.
///
/// A monster does not get knocked into the air and it does not get dragged to
/// the end of somebody's arm, but the *decision* to throw a knock-up at it
/// should still be a decision -- otherwise half of every kit is dead weight in
/// a hunt and the two halves of the game stop teaching each other anything.
/// So a piece of crowd control is offered here and the creature answers with
/// what it is willing to take. See `Monster::take_control`.
#[derive(Clone, Copy, Debug, Default)]
pub struct Control {
    /// Upward speed the move would have given a fighter. Anything above zero
    /// reads as a knock-up.
    pub launch: Fx,
    /// Frames the move would have held a fighter for.
    pub grabs: u16,
    /// A slow, as the fighters' own `Player::slow` takes it.
    pub slow_frames: u16,
    pub slow_mul: Fx,
}

impl Control {
    pub fn launching(launch: Fx) -> Control {
        Control {
            launch,
            ..Control::default()
        }
    }

    pub fn grabbing(frames: u16) -> Control {
        Control {
            grabs: frames,
            ..Control::default()
        }
    }

    pub fn slowing(frames: u16, mul: Fx) -> Control {
        Control {
            slow_frames: frames,
            slow_mul: mul,
            ..Control::default()
        }
    }

    pub fn is_anything(&self) -> bool {
        self.launch.raw() > 0 || self.grabs > 0 || self.slow_frames > 0
    }
}

/// **A blow arriving at a creature, as a guard sees it**: where it comes
/// from, whether it is a guard breaker, and who threw what. Handed to a
/// species' guard (`species::FightDecl::guard`) before the blow is taken, at
/// every place in the world a blow reaches a creature -- a swing, a Reaver's
/// copy or her recall, an effect, a beam, a bolt, a gust, a stone's
/// shrapnel -- because a guard that stopped some kinds of attack and not
/// others would be worse than none (`state::guard_against`, the fighters'
/// half of the same rule).
#[derive(Clone, Copy, Debug)]
pub struct Blow {
    /// The point the hit comes from: the attacker's body, the shadow that
    /// threw a copy, the effect or the bolt where it is. The same point the
    /// fighters' `guard_against` is handed for the same kind of attack.
    pub from: V3,
    /// **A guard breaker**: the versus rule's own flag, and nothing else.
    pub unblockable: bool,
    /// The fighter whose blow it is, or `state::NOBODY` for nobody's.
    pub who: u8,
    /// Their class, and the move -- [`Blow::NO_MOVE`] when the blow is not
    /// one of a class's moves (a stone's shrapnel).
    pub class: crate::Class,
    pub kind: u8,
}

impl Blow {
    /// Not one of a class's moves.
    pub const NO_MOVE: u8 = u8::MAX;
}

/// What a creature's guard made of a [`Blow`].
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Guarded {
    /// No guard, or not against this: it lands as it would have.
    Lands,
    /// Stopped by the guard: nothing reaches the body.
    Blocked,
    /// Stopped inside the parry window: nothing reaches the body, and the
    /// species makes of the parry what it will (the Mantis's counter).
    Parried,
    /// A guard breaker through a raised guard: it lands, and the guard is
    /// broken.
    Broke,
}

impl Guarded {
    /// The guard stopped it: no damage, no control, no mark.
    pub const fn turned(self) -> bool {
        matches!(self, Guarded::Blocked | Guarded::Parried)
    }
}

/// What the creature actually took.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Took {
    pub stumbled: bool,
    pub rooted: bool,
    pub slowed: bool,
}

impl Took {
    pub fn anything(&self) -> bool {
        self.stumbled || self.rooted || self.slowed
    }
}

// ---------------------------------------------------------------------------
// The creature
// ---------------------------------------------------------------------------

/// What the creature is allowed to know about one fighter.
///
/// A deliberately small window. The control algorithm is handed this and
/// nothing else -- no buttons, no action state, no frame counters -- so
/// "it reads your inputs" is not a thing that can accidentally become true.
#[derive(Clone, Copy, Debug, Default)]
pub struct Quarry {
    pub pos: V3,
    pub vel: V3,
    pub alive: bool,
    /// Standing on the creature.
    pub aboard: bool,
    /// In hitstun, staggered, or rooted: unable to act for the moment. What
    /// the follow-up appetite reads -- a sweep that staggers is a sweep that
    /// sets up the bite, and this is how the animal knows to throw it.
    pub stunned: bool,
}

/// What a creature's senses allow it this frame (bestiary P5): which fighters
/// its species' perception filter lets the glance sample, and the noise it
/// would attend to if it perceives nobody.
///
/// Worked out by the world, which has the scene a filter needs -- line of
/// sight is asked through `aim` -- and handed in beside the [`Quarry`], so the
/// brain still receives positions and velocities and nothing else.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Senses {
    pub perceived: [bool; crate::state::MAX_PLAYERS],
    pub heard: Option<Heard>,
}

impl Senses {
    /// It perceives everybody and hears nothing: the Ridgeback's, and the
    /// glance as it was before there were filters.
    pub const ALL: Senses = Senses {
        perceived: [true; crate::state::MAX_PLAYERS],
        heard: None,
    };
}

/// A noise the creature attends to: where, and who made it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Heard {
    pub at: V3,
    /// The fighter who made it, or `noise::NOBODY`.
    pub who: u8,
}

/// **What a species' brain hooks may read** (`species::FightDecl`'s
/// `appetite`, `prowl_to`, `commit`): the fighters as the brain sees them --
/// positions and velocities, never buttons -- the ground as it stands, its
/// floor hazards included, and the hunt's lore. The same small window the
/// shared brain has, with the floor added, so a species' own terms cannot
/// read an input either.
#[derive(Clone, Copy)]
pub struct Mind<'a> {
    pub quarry: &'a [Quarry],
    pub ground: &'a Terrain,
    pub lore: &'a crate::lore::Lore,
}

/// Where its attention is, and what it remembers seeing.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Brain {
    /// The last sample it took, and when the next one is due.
    pub seen: V3,
    pub seen_vel: V3,
    /// Whether the target could act, as of the last glance.
    pub seen_stunned: bool,
    pub target: u8,
    pub glance_left: u16,
    /// The pause between moves. Without one it is a chain gun.
    pub think_left: u16,
    pub last_move: u8,
    /// Frames of variety penalty left on `last_move`.
    pub repeat_left: u16,
    /// Per-move lockout. Zero means available. Room for the species with the
    /// most moves; the rest stay zero.
    pub cooldown: [u16; MAX_MOVES],
    /// The move in progress plays its clip the other way round. Set when a
    /// move that `mirrors_to_target_side` is chosen, from which side the
    /// target was on -- the Ridgeback's tail goes where the person is rather
    /// than always to the right.
    pub mirror: bool,
    /// Deterministic, advanced only from inside the tick, and part of the
    /// snapshot -- so a rollback replays the same choices.
    pub rng: u32,
    /// **Frames left of the moment it takes to notice you**, at the start of a
    /// hunt. It stands its ground and turns to face you, and throws nothing;
    /// a hit wakes it at once. See `Species::hunt_grace`.
    pub grace: u16,
    /// **Where the move in progress is aimed at, on the floor**, in
    /// centimetres: chosen when it commits and held. What a `lobbed` move's
    /// volume lands on -- the Mireback's glob and its belly flop -- and where a
    /// move that `stops_at_aim` stops travelling: the tongue, at the first
    /// solid on its line. See `species::MoveDecl`. Zero, and never read, for a
    /// move with neither flag, so it is only hashed when it is not.
    pub aim: [i16; 2],
}

impl Default for Brain {
    fn default() -> Brain {
        Brain {
            seen: V3::ZERO,
            seen_vel: V3::ZERO,
            seen_stunned: false,
            target: 0,
            glance_left: 0,
            think_left: 0,
            last_move: NO_PART,
            repeat_left: 0,
            cooldown: [0; MAX_MOVES],
            mirror: false,
            // Any odd seed. Xorshift is stuck at zero, and one is as good a
            // starting point as any other.
            rng: 0x2545_F491,
            grace: 0,
            aim: [0; 2],
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Monster {
    /// Which kind of creature: the table everything below is read against.
    pub species: SpeciesId,
    /// **How clever it is this hunt**: 0 as tuned, up to
    /// [`crate::temper::HIGHEST`]. Read through the four tempered accessors
    /// below, never past them. See [`crate::temper`].
    pub temper: u8,
    pub pos: V3,
    /// Steered facing, in turns.
    pub yaw: Fx,
    /// Turns per second. Having a rate at all is what gives the creature mass:
    /// it cannot reverse a swing instantly, so it overshoots, and the overshoot
    /// is the window a player cuts back across its nose for.
    pub yaw_rate: Fx,
    /// Forward speed along its facing. It does not strafe; a quadruped that
    /// could would make its turn limit meaningless.
    pub speed: Fx,
    pub health: i32,
    /// Fills with damage to either weak point. Full, it goes over.
    pub poise: i32,
    /// **Damage taken recently**, bleeding away every frame.
    ///
    /// This is the whole of the crowd-control model and half of the interrupt
    /// one. A creature this size ignores a knock-up thrown at it cold; hurt it
    /// hard enough in the last couple of seconds and its footing is bad enough
    /// that the same knock-up takes it off balance. Both thresholds fall as its
    /// health does, so a hunt starts methodical and ends frantic. See
    /// `docs/design/monsters.md` §4.
    pub strain: i32,
    /// Health of each breakable part, in the order the species lists them
    /// (`Species::breakables`). Only breakable parts have one: see
    /// `beast::MAX_BREAKABLE`. Read it through [`Monster::part_health`].
    pub breaks: [i32; MAX_BREAKABLE],
    pub doing: Doing,
    pub brain: Brain,
    /// The current active window has already connected.
    pub hit_used: bool,
    /// Where it is in its stride, as a fraction of one cycle in 16.16's
    /// fractional bits -- the same accumulator the fighters carry, for the same
    /// reason: a gait on a fixed cadence skates the moment the body moves at
    /// any other speed.
    ///
    /// In the snapshot, unlike a fighter's, because the creature's legs are
    /// **collision geometry**: where a foot is decides what a player walks into.
    pub stride: u16,
    /// A free-running clock, for the parts of the idle that have nothing to do
    /// with where it is going. Breathing, mostly.
    pub beat: u16,
    /// Frames of slow left, and how much of its speed the slow leaves it.
    pub slowed: u16,
    pub slow_mul: Fx,
    /// Frames it is pinned in place. What a grab does to something too big to
    /// drag.
    pub rooted: u16,
    /// The Reaver's marks, and the clock that fades them -- the same pair a
    /// fighter carries (`state::Player::marks`). The shadow works a
    /// Ridgeback's flank from range and she crosses to cash on a leg. The same
    /// cap as a fighter's for now; whether a body this big wants its own is an
    /// open question in `docs/design/shadow-reaver-v2.md`.
    pub marks: u8,
    pub mark_clock: u16,
    /// **Four words the species keeps on the body**, for the hooks that are
    /// handed the creature and nothing else: what its hide is worth this
    /// frame (`FightDecl::hide`) and what a hit does to it besides the ladder
    /// (`FightDecl::struck`). The hit path reaches a creature from a dozen
    /// places that have no world to hand, so state those two read lives here
    /// rather than in the hunt's lore. Its file says what each word means;
    /// zero for a species with no such hooks, and only hashed when not zero.
    /// The Mireback's coat and its throat sac's poise.
    pub own: [i32; 4],
}

impl Monster {
    /// A creature of this species, at full health, standing at the origin.
    pub fn new(species: SpeciesId) -> Monster {
        let sp = species.get();
        Monster {
            species,
            temper: 0,
            pos: V3::ZERO,
            yaw: Fx::ZERO,
            yaw_rate: Fx::ZERO,
            speed: Fx::ZERO,
            health: sp.health(),
            poise: 0,
            strain: 0,
            breaks: [sp.part_health(); MAX_BREAKABLE],
            doing: Doing::Prowl,
            brain: Brain::default(),
            hit_used: false,
            stride: 0,
            beat: 0,
            slowed: 0,
            slow_mul: Fx::ONE,
            rooted: 0,
            marks: 0,
            mark_clock: 0,
            own: [0; 4],
        }
    }

    /// Its species' table.
    pub fn sp(&self) -> &'static Species {
        self.species.get()
    }

    // The four brain knobs a temper overrides (`crate::temper`). The brain
    // reads them here rather than off the species, so a tempered hunt is the
    // same code reading different numbers.

    /// Frames between glances, after its temper.
    pub fn glance_frames(&self) -> u16 {
        // Its species' say first, from its own state (`FightDecl::glance`):
        // the Pair glance quicker with two hunters to watch.
        let own = match self.sp().fight.glance {
            Some(f) => f(self),
            None => self.sp().glance_frames(),
        };
        crate::temper::glance(own, self.temper)
    }

    /// Lead on the target, after its temper.
    pub fn lead(&self) -> Fx {
        crate::temper::lead(self.sp().lead(), self.temper)
    }

    /// Decisiveness, after its temper.
    pub fn decisiveness(&self) -> i32 {
        crate::temper::decisiveness(self.sp().decisiveness(), self.temper)
    }

    /// How far its strain thresholds fall by the time it is nearly dead,
    /// after its temper.
    pub fn strain_desperation(&self) -> i32 {
        crate::temper::despair(self.sp().strain_desperation(), self.temper)
    }

    /// A part's health. A part that cannot break is always at full: it has no
    /// health of its own to lose.
    pub fn part_health(&self, part: usize) -> i32 {
        match self.sp().break_slot(part) {
            Some(slot) => self.breaks[slot],
            None => self.sp().part_health(),
        }
    }

    /// One mark from the Reaver's shadow. See `shadow::mark`, which is the
    /// same rule for a fighter.
    pub fn mark(&mut self) {
        self.marks = self.marks.saturating_add(1).min(t::mark_cap());
        self.mark_clock = t::mark_fade();
    }

    /// Spend every mark, and say how many there were.
    pub fn spend_marks(&mut self) -> u8 {
        let marks = self.marks;
        self.marks = 0;
        self.mark_clock = 0;
        marks
    }

    /// On its side, with its options gone.
    ///
    /// The creature's answer to `state::Player::disabled`, and drawn on the
    /// same line: a topple is the long window the whole climb exists to earn,
    /// and a flinch is the cheap one that happens whenever it is hit hard
    /// enough. A stumble counts -- it has to be earned, by breaking a foot or
    /// by landing crowd control on a creature that was already hurting.
    pub fn disabled(&self) -> bool {
        matches!(self.doing, Doing::Toppled { .. } | Doing::Stumble { .. })
    }

    pub fn alive(&self) -> bool {
        self.health > 0
    }

    /// A foot with no health left. Breaking one drops that corner of the animal
    /// for good and puts it on its knee for a moment.
    pub fn broken(&self, part: usize) -> bool {
        self.sp()
            .break_slot(part)
            .is_some_and(|slot| self.breaks[slot] <= 0)
    }

    /// How many legs are gone at the front, and how many at the back.
    pub fn lameness(&self) -> (i32, i32) {
        let mut front = 0;
        let mut rear = 0;
        for leg in self.sp().legs {
            if self.broken(leg.foot) {
                if leg.front {
                    front += 1;
                } else {
                    rear += 1;
                }
            }
        }
        (front, rear)
    }

    /// Net lean to one side, in broken legs: negative is to its left.
    fn list(&self) -> i32 {
        self.sp()
            .legs
            .iter()
            .filter(|leg| self.broken(leg.foot))
            .map(|leg| leg.side)
            .sum()
    }
}

// ---------------------------------------------------------------------------
// The pose
// ---------------------------------------------------------------------------

impl Monster {
    /// What the creature looks like this frame.
    ///
    /// A baked clip, chosen by what it is doing and read at the phase it is at,
    /// with two layers added over it: the gait, and whatever its broken legs
    /// are doing to its stance. Both additive, so a creature that is biting
    /// while limping is one pose rather than a special case.
    ///
    /// Everything it reads is in the snapshot, so this is reproducible under
    /// rollback -- which is the only property the pose has to have, and the
    /// reason the clips are a table rather than a solver.
    pub fn pose(&self) -> Pose {
        let mut p = self.clip_pose();
        let lame = self.lame_layer();
        p = p.over(&lame, Fx::ONE);
        if self.doing.free() {
            p = p.over(&self.tracking_layer(), Fx::ONE);
        }
        // Its species' own say, last (`FightDecl::repose`).
        match self.sp().fight.repose {
            Some(f) => f(self, p),
            None => p,
        }
    }

    /// The baked clip underneath everything else.
    fn clip_pose(&self) -> Pose {
        let sp = self.sp();
        // Its posture first, for a species whose posture is not its speed
        // (`FightDecl::clip`).
        if let Some(posed) = sp.fight.clip.and_then(|f| f(self)) {
            return posed;
        }
        let stock = sp.stock;
        match self.doing {
            Doing::Dead => beast::sample(sp, stock.dead, Fx::ONE),
            Doing::Toppled { left } => {
                beast::sample(sp, stock.topple, through(left, sp.topple_frames()))
            }
            Doing::Stumble { left, .. } => {
                beast::sample(sp, stock.stumble, through(left, sp.stumble_frames()))
            }
            Doing::Flinch { left } => {
                beast::sample(sp, stock.flinch, through(left, sp.flinch_frames()))
            }
            Doing::Prowl => self.locomotion(),
            _ => {
                let kind = self.doing.attacking().unwrap_or(0);
                let (which, at) = phase(sp, self.doing);
                let decl = sp.moves[(kind as usize).min(sp.moves.len() - 1)];
                let mut posed = beast::sample_phase(sp, decl.clip, which, at);
                if self.brain.mirror {
                    posed = posed.mirrored(sp.mirror);
                }
                if let Some(knob) = decl.scaled_by {
                    // The one kind of clip whose *violence* is a gameplay
                    // number rather than a look: it is what decides whether a
                    // braced rider stays on. Everything else about the
                    // animation is content; this is a knob. See
                    // `species::MoveDecl::scaled_by`.
                    return Pose::rest(sp.bones.len()).over(&posed, sp.own_fx(knob as usize));
                }
                posed
            }
        }
    }

    /// Standing, walking or running, blended by how fast it is going.
    ///
    /// Indexed by **ground covered** rather than by time. The cycles are
    /// sampled at the same stride phase before being blended, which is what
    /// stops a creature accelerating out of a walk from planting two feet at
    /// once.
    fn locomotion(&self) -> Pose {
        let sp = self.sp();
        let phase = Fx::from_raw(self.stride as i32);
        let breath = Fx::from_raw(self.beat as i32);
        let idle = beast::sample(sp, sp.stock.idle, breath);
        let speed = self.speed.abs();
        if speed.raw() <= 0 {
            return idle;
        }
        let walk = beast::sample(sp, sp.stock.walk, phase);
        // Never zero, so the division below is a division. `Fx::from_raw(1)` is
        // the smallest number 16.16 has, not a speed anybody chose.
        let cruise = sp.walk().max(SMALLEST);
        if speed.raw() <= cruise.raw() {
            return idle.blend(&walk, speed.div(cruise).clamp(Fx::ZERO, Fx::ONE));
        }
        let gallop = beast::sample(sp, sp.stock.gallop, phase);
        let top = sp.gallop().max(cruise.add(SMALLEST));
        let at = speed
            .sub(cruise)
            .div(top.sub(cruise))
            .clamp(Fx::ZERO, Fx::ONE);
        walk.blend(&gallop, at)
    }

    /// What its broken legs do to how it stands.
    ///
    /// A corner with nothing under it drops, the body leans onto the legs that
    /// are left, and the useless foot folds up rather than punching through the
    /// floor. This is the *permanent* consequence -- the stumble is the moment
    /// it happens; this is what the animal looks like for the rest of the
    /// fight, and it is what brings the back low enough to matter.
    fn lame_layer(&self) -> Pose {
        let sp = self.sp();
        let (front, rear) = self.lameness();
        if front == 0 && rear == 0 {
            return Pose::rest(sp.bones.len());
        }
        let drop = sp.leg_drop();
        let mut p = Pose::rest(sp.bones.len());
        let f = Fx::from_int(front);
        let r = Fx::from_int(rear);
        // The **average** of the two ends rather than the sum. The hips are one
        // point, and how much lower the front is than the back is the pitch
        // below saying so -- adding both ends into the height as well drops the
        // whole animal through the floor once three feet are gone.
        let sunk = drop.mul(f.add(r)).mul(Fx::ratio(1, 2));
        p.hips.y = sunk.neg();
        // Nose down when the front is gone, tail down when the back is. The
        // rig's pitch is positive nose-*up*, so the subtraction runs the other
        // way round to the one that reads naturally here. The root is bone
        // zero: parents come before children, so nothing else can be.
        let pitch = sp.leg_pitch().mul(r.sub(f));
        p.bone[0].x = pitch;
        p.bone[0].z = sp.leg_roll().mul(Fx::from_int(self.list()));
        for leg in sp.legs {
            if !self.broken(leg.foot) {
                // **A sound leg on a lowered animal bends to meet the floor.**
                // The hips came down and this leg did not get shorter, so it
                // folds at both joints by the angle that takes exactly the
                // drop out of its height -- what a real leg does under a body
                // that has sagged onto it. Without this the legs that are
                // left stand through the ground, and with the legs a metre
                // longer than they were that is a metre of animal buried.
                // The drop this end felt is the hips' drop corrected by the
                // pitch: nose-up raises the front and lowers the rear. The
                // lever is how far forward of the hips the leg hangs: its
                // hip's rest offset and every bone's above it, short of the
                // root.
                let mut lever = Fx::ZERO;
                let mut b = leg.hip;
                while sp.bones[b].parent != beast::NO_PARENT {
                    lever = lever.add(sp.rest(b).x);
                    b = sp.bones[b].parent;
                }
                let felt = sunk.sub(crate::math::turns_to_radians(pitch).mul(lever));
                let len = sp.rest(leg.knee).len().add(sp.shape(leg.foot).min.y.neg());
                let c = crate::math::crouch_turns(felt, len);
                // Folded the way this leg's joint goes -- the same rule the
                // break below uses, and the same sign convention.
                let (hip, knee) = if leg.front {
                    (c.neg(), c.add(c))
                } else {
                    (c, c.add(c).neg())
                };
                p.bone[leg.hip].x = p.bone[leg.hip].x.add(hip);
                p.bone[leg.knee].x = p.bone[leg.knee].x.add(knee);
                continue;
            }
            // The joint above a broken foot folds. Which way depends on which
            // end it is: a foreleg buckles forward at the knee, a hindleg back
            // at the hock, and that difference is most of what a limp looks
            // like from across the arena.
            let fold = if leg.front {
                sp.leg_fold()
            } else {
                sp.leg_fold().neg()
            };
            p.bone[leg.hip].x = p.bone[leg.hip].x.add(sp.leg_buckle().mul(fold).neg());
            p.bone[leg.knee].x = p.bone[leg.knee].x.add(fold);
        }
        p
    }

    /// The neck turning to keep the target in view.
    ///
    /// Only while it is free -- a committed move has already decided where it
    /// is pointed, and letting the head wander during one would be the
    /// telegraph lying. Split across the neck's bones so it curves rather than
    /// hinging, which is the whole difference between a neck and a door.
    fn tracking_layer(&self) -> Pose {
        let sp = self.sp();
        let to = V3::new(
            self.brain.seen.x.sub(self.pos.x),
            Fx::ZERO,
            self.brain.seen.z.sub(self.pos.z),
        );
        if to.flat_len().raw() <= 0 || sp.neck.is_empty() {
            return Pose::rest(sp.bones.len());
        }
        let want = crate::math::atan2_turns(to.z, to.x);
        let error = crate::math::wrap_turns(want.sub(self.yaw));
        let reach = sp.head_track();
        // Shared equally down the neck, so the answer curves rather than
        // hinging. The divisor is how many bones there are, not a number.
        let each = error
            .clamp(reach.neg(), reach)
            .div(Fx::from_int(sp.neck.len() as i32));
        let mut p = Pose::rest(sp.bones.len());
        for bone in sp.neck {
            p.bone[*bone].y = each;
        }
        p
    }

    /// Every bone, placed in the world.
    pub fn rig(&self) -> Rig {
        let mut rig = Rig::build(self.sp(), self.pos, self.yaw, &self.pose());
        // What of it is under the floor this frame, for a species with a
        // body that is not always there (`FightDecl::presence`).
        if let Some(presence) = self.sp().fight.presence {
            let here = presence(self, &rig);
            rig.buried = here.buried;
            rig.unmountable = here.unmountable;
            rig.passable = here.passable;
        }
        rig
    }

    /// Body-space position of a world point in a part's own frame -- what a
    /// rider holds on to, so that the bone moving under them moves them with it.
    pub fn rest_frame(&self, part: usize, world: V3) -> V3 {
        self.rig().world_to_part(part, world)
    }

    /// The world position of a point held in a part's own frame.
    pub fn world_of(&self, part: usize, rest: V3) -> V3 {
        self.rig().part_to_world(part, rest)
    }

    /// The mountable part a body at `world` is standing on, if any.
    ///
    /// Mounting is landing: the feet have to be within `mount_snap` of a
    /// mountable top face and over its footprint. No button, because a surface
    /// is a surface.
    ///
    /// **The same question answers "are you still on it".** How far you may
    /// overhang an edge is `edge_grace`, as a fraction of your own width, and
    /// it is applied here rather than in a second test -- two rules for
    /// standing on something is how a rider ends up hovering off the side.
    pub fn surface_under(&self, world: V3, body_radius: Fx) -> Option<(usize, Fx)> {
        self.rig().surface_under(world, body_radius)
    }
}

impl Rig {
    /// Is this part's top face level enough to stand on? Always, for a
    /// species that does not say (`FightDecl::steepest`).
    fn standable(&self, part: usize) -> bool {
        // **A top that sheds is never a floor** (`beast::Shape::sheds`).
        if self.species.parts.get(part).is_some_and(|p| p.shape.sheds) {
            return false;
        }
        match self.species.fight.steepest {
            Some(k) => self.of(part).rot.r[1].y.raw() >= self.species.own_raw(k as usize),
            None => true,
        }
    }

    /// **The first top face a ray meets from above** (aiming A3): how far
    /// along it, for a part you could stand on. Seen from above the top of a
    /// mountable part is a *place* -- a rider aiming at their own feet on a
    /// shell means the shell, not the valley floor twenty metres under it --
    /// and met from below, or on a side, the creature is still a body, off
    /// the aiming ray. Asked by `aim` only; the geometry is here because the
    /// articulation is.
    pub fn top_along(&self, from: V3, dir: V3) -> Option<Fx> {
        let mut best: Option<Fx> = None;
        for (index, part) in self.species.parts.iter().enumerate() {
            if !part.shape.mountable || !self.boardable(index) || !self.standable(index) {
                continue;
            }
            if self.species.fight.rolls_over && self.of(index).rot.r[1].y.raw() <= 0 {
                continue;
            }
            let frame = self.of(index);
            let d = frame.rot.unapply(dir);
            // From above: the ray is going down the part's own up.
            if d.y.raw() >= 0 {
                continue;
            }
            let o = frame.world_to_local(from);
            let sh = self.species.shape(index);
            let Some(dist) = crate::math::ray_hits_box(o, d, sh.min, sh.max) else {
                continue;
            };
            let y = o.y.add(d.y.mul(dist));
            if y.sub(sh.max.y).abs().raw() > crate::arena::SKIN.raw() {
                continue;
            }
            if best.is_none_or(|seen| dist.raw() < seen.raw()) {
                best = Some(dist);
            }
        }
        best
    }

    /// The highest top face you could stand on under `world`, no higher than
    /// it, as a height in the world: where a thing put down at that point
    /// lands, aboard.
    pub fn footing_under(&self, world: V3) -> Option<Fx> {
        let (part, top) =
            self.surface_within(world, Fx::ZERO, world.y.max(Fx::ZERO), crate::arena::SKIN)?;
        let local = self.world_to_part(part, world);
        Some(self.part_to_world(part, V3::new(local.x, top, local.z)).y)
    }

    /// See [`Monster::surface_under`]. Lives on the rig because everything it
    /// needs is a bone transform, and the callers that already hold one should
    /// not have to rebuild the skeleton to ask.
    pub fn surface_under(&self, world: V3, body_radius: Fx) -> Option<(usize, Fx)> {
        self.surface_within(world, body_radius, t::mount_snap(), t::mount_snap())
    }

    /// The mountable part whose top face is somewhere between `below` beneath a
    /// point and `above` over it.
    ///
    /// Two callers with two different questions. Landing asks about a band as
    /// deep as it is tall -- are my feet on this. **Stepping up asks about a
    /// band that is deep by a whisker and tall by a whole step**, which is the
    /// difference between walking up a kerb and levitating onto a roof.
    ///
    /// Splitting them fixed a real dead end: the step-up probe used to sample
    /// one point a full step above the feet and ask whether *that* was standing
    /// on something, so it only ever found a surface sitting at exactly one
    /// step up. A rider walking from the haunch to the barrel -- four
    /// centimetres, not a step -- was shoved back by the barrel's own side
    /// every frame and never got on, and the climb stopped at the hips.
    pub fn surface_within(
        &self,
        world: V3,
        body_radius: Fx,
        below: Fx,
        above: Fx,
    ) -> Option<(usize, Fx)> {
        let lip = body_radius.mul(t::edge_grace());
        let mut best: Option<(usize, Fx, Fx)> = None;
        for (index, part) in self.species.parts.iter().enumerate() {
            if !part.shape.mountable || !self.boardable(index) {
                continue;
            }
            // **A face pointing at the floor is not a surface**, for a species
            // that rolls onto its back (`FightDecl::rolls_over`).
            if self.species.fight.rolls_over && self.of(index).rot.r[1].y.raw() <= 0 {
                continue;
            }
            // **Nor is a face too steep to stand on**, for a species that
            // says how steep (`FightDecl::steepest`).
            if !self.standable(index) {
                continue;
            }
            let sh = self.species.shape(index);
            let local = self.world_to_part(index, world);
            let over = local.x.raw() > sh.min.x.sub(lip).raw()
                && local.x.raw() < sh.max.x.add(lip).raw()
                && local.z.raw() > sh.min.z.sub(lip).raw()
                && local.z.raw() < sh.max.z.add(lip).raw();
            if !over {
                continue;
            }
            // How far the face is *above* the point asking.
            let rise = sh.max.y.sub(local.y);
            if rise.raw() > above.raw() || rise.neg().raw() > below.raw() {
                continue;
            }
            // Highest *in the world*, not highest in the bone's own frame: on a
            // skeleton those are different questions the moment anything is
            // pitched, and taking the local answer put riders on whichever part
            // happened to be authored tallest.
            let height = self
                .part_to_world(index, V3::new(local.x, sh.max.y, local.z))
                .y;
            // **A face under the floor is nobody's to stand on**, for a
            // species with a body that goes under it (`FightDecl::steepest`
            // says it has faces that are not always floors).
            if self.species.fight.steepest.is_some() && height.raw() < 0 {
                continue;
            }
            if best.is_none_or(|(_, _, seen)| height.raw() > seen.raw()) {
                best = Some((index, sh.max.y, height));
            }
        }
        best.map(|(i, top, _)| (i, top))
    }
}

// ---------------------------------------------------------------------------
// Collision, bone by bone
// ---------------------------------------------------------------------------

/// What resolving a body against the creature did to it.
#[derive(Clone, Copy, Debug)]
pub struct Contact {
    pub pos: V3,
    /// The part it came to rest on top of, if any.
    pub landed: Option<usize>,
    /// It was pushed sideways -- it walked into the animal.
    pub shoved: bool,
}

/// Distance from a point to a box in the horizontal plane. Zero inside.
fn flat_gap(x: Fx, z: Fx, min: V3, max: V3) -> Fx {
    let dx = min.x.sub(x).max(x.sub(max.x)).max(Fx::ZERO);
    let dz = min.z.sub(z).max(z.sub(max.z)).max(Fx::ZERO);
    V3::new(dx, Fx::ZERO, dz).flat_len()
}

/// Signed push needed to leave a span by the nearer edge. Same rule as the
/// arena's, and for the same reason: least penetration is the only resolution
/// that does not teleport a body through a thin box.
fn out_of(v: Fx, lo: Fx, hi: Fx) -> Fx {
    let low = lo.sub(v);
    let high = hi.sub(v);
    if high.abs().raw() < low.abs().raw() {
        high
    } else {
        low
    }
}

impl Monster {
    /// Push a body out of the creature's solid parts.
    ///
    /// Every part is an axis-aligned box **in its own bone's frame**, so this
    /// is the arena's collision rule run once per bone: transform the body into
    /// the bone's frame, resolve against the box, transform back. One rule,
    /// eighteen frames of reference.
    ///
    /// The body stays a vertical cylinder in the world while the box does not,
    /// so a heavily rotated bone -- a foreleg at the top of a stomp -- resolves
    /// a little conservatively. That is the same approximation the single-frame
    /// version made for pitch, and it is the right one for a blockout: being
    /// pushed out of a leg slightly early is invisible, and being pulled
    /// through one is not.
    pub fn resolve(&self, world: V3, body_radius: Fx, body_height: Fx) -> Contact {
        self.rig().resolve(world, body_radius, body_height)
    }

    /// Is a point inside the creature's solid parts, padded outward?
    ///
    /// For the camera. The rig's rule is that level geometry never gets between
    /// the eye and the fighter and that the arm pulls *in* rather than swinging
    /// away; a creature this size is level geometry for as long as it is
    /// standing between the two.
    pub fn contains(&self, world: V3, pad: Fx) -> bool {
        let rig = self.rig();
        let sp = self.sp();
        sp.parts.iter().enumerate().any(|(index, part)| {
            if !part.shape.solid || !rig.blocks(index) {
                return false;
            }
            let sh = sp.shape(index);
            let p = rig.world_to_part(index, world);
            p.x.raw() > sh.min.x.sub(pad).raw()
                && p.x.raw() < sh.max.x.add(pad).raw()
                && p.y.raw() > sh.min.y.sub(pad).raw()
                && p.y.raw() < sh.max.y.add(pad).raw()
                && p.z.raw() > sh.min.z.sub(pad).raw()
                && p.z.raw() < sh.max.z.add(pad).raw()
        })
    }

    /// World height of the highest solid part top over a point that is **no
    /// higher than `below`**, or zero.
    ///
    /// The companion to `arena::ground_under`, and used for the same thing: the
    /// camera treats a surface as something to rest on rather than something to
    /// dodge, and the creature's back is a surface. The ceiling is the point:
    /// the camera asks for the surface under the fighter's own feet, and a
    /// creature passing *over* the eye -- a bird a few metres up -- is not a
    /// floor to put the camera on (galewing.md §6).
    pub fn top_under(&self, world: V3, below: Fx) -> Fx {
        let rig = self.rig();
        let sp = self.sp();
        let mut best = Fx::ZERO;
        for (index, part) in sp.parts.iter().enumerate() {
            if !part.shape.solid || !rig.blocks(index) {
                continue;
            }
            let sh = sp.shape(index);
            let p = rig.world_to_part(index, world);
            let over = p.x.raw() > sh.min.x.raw()
                && p.x.raw() < sh.max.x.raw()
                && p.z.raw() > sh.min.z.raw()
                && p.z.raw() < sh.max.z.raw();
            if !over {
                continue;
            }
            let top = rig.part_to_world(index, V3::new(p.x, sh.max.y, p.z)).y;
            if top.raw() > best.raw() && top.raw() <= below.raw() {
                best = top;
            }
        }
        best
    }
}

impl Rig {
    /// See [`Monster::resolve`].
    pub fn resolve(&self, world: V3, body_radius: Fx, body_height: Fx) -> Contact {
        let mut at = world;
        let mut landed = None;
        let mut shoved = false;

        for (index, part) in self.species.parts.iter().enumerate() {
            let part = part.shape;
            if !part.solid || !self.blocks(index) {
                continue;
            }
            let sh = self.species.shape(index);
            let frame = self.of(index);
            let mut p = frame.world_to_local(at);

            let min_x = sh.min.x.sub(body_radius);
            let max_x = sh.max.x.add(body_radius);
            let min_z = sh.min.z.sub(body_radius);
            let max_z = sh.max.z.add(body_radius);
            let head = p.y.add(body_height);
            let inside = p.x.raw() > min_x.raw()
                && p.x.raw() < max_x.raw()
                && p.z.raw() > min_z.raw()
                && p.z.raw() < max_z.raw()
                && head.raw() > sh.min.y.raw()
                && p.y.raw() < sh.max.y.raw();
            if !inside {
                continue;
            }

            let px = out_of(p.x, min_x, max_x);
            let pz = out_of(p.z, min_z, max_z);
            let up = sh.max.y.sub(p.y);
            let down = head.sub(sh.min.y);
            let vertical = up.min(down).abs();

            // **A tread you could step onto is a floor, not a wall.** Least
            // penetration alone is not enough here: the creature's mountable
            // parts overlap so the climb has no seams, and where two of them
            // meet the taller one is a lip a few centimetres proud of the
            // shorter. A body standing in that lip is nearer its side than its
            // top by a hair, so least penetration shoves them backwards --
            // every frame, forever.
            //
            // Bounded by `mount_snap` rather than by `step_up`, and the
            // difference is the whole of getting it right: a *landing*
            // tolerance is a few centimetres and is what a seam between two
            // treads measures, while a step is most of a metre and a rider
            // walking into the side of one should be stopped by it rather than
            // hoisted on top. Using the step here lifted a rider off the tail
            // onto the haunch mid-sweep and dropped them off the far side.
            let step = part.mountable
                && self.boardable(index)
                && self.standable(index)
                && up.raw() >= 0
                && up.raw() <= t::mount_snap().raw();
            // **Never pressed down**, for a species whose parts come down on
            // bodies (`FightDecl::lands_on_bodies`): a body under the
            // Mireback's belly as a flop lands -- on the floor, or on a slag
            // mound -- is shoved out sideways, where least penetration would
            // push its feet through whatever it stood on and nothing pushes
            // them back. The Ridgeback does not
            // say so, and keeps its pinned hunts: whether it should is a
            // person's call (`docs/design/hazards.md` §8).
            let into_floor = self.species.fight.lands_on_bodies && up.raw() > down.raw();
            if into_floor {
                // Out the way it is from the creature's middle, all the way
                // out of this box: every box it is in pushes it the same way,
                // so a body the creature landed on comes out of the side of
                // it rather than being passed from box to box inside.
                let from_middle = at.sub(self.origin);
                let away = V3::new(from_middle.x, Fx::ZERO, from_middle.z);
                let away = if away.flat_len().raw() > 0 {
                    crate::math::wide_normalized(away)
                } else {
                    self.facing.apply(V3::new(Fx::ONE, Fx::ZERO, Fx::ZERO))
                };
                let d = frame.rot.unapply(away);
                let exit = |v: Fx, dv: Fx, lo: Fx, hi: Fx| -> Fx {
                    if dv.raw() > 0 {
                        hi.sub(v).div(dv)
                    } else if dv.raw() < 0 {
                        lo.sub(v).div(dv)
                    } else {
                        Fx::MAX
                    }
                };
                let t = exit(p.x, d.x, min_x, max_x).min(exit(p.z, d.z, min_z, max_z));
                if t.raw() < Fx::MAX.raw() {
                    p.x = p.x.add(d.x.mul(t));
                    p.z = p.z.add(d.z.mul(t));
                    shoved = true;
                }
            } else if step
                // A top that sheds is never stood on, and a body under it is
                // never pressed into the floor: it goes out of the side, the
                // way it came in (`beast::Shape::sheds`).
                || (!part.sheds
                    && vertical.raw() <= px.abs().raw()
                    && vertical.raw() <= pz.abs().raw()
                    // A face too steep to stand on is never stood on by
                    // least penetration either: it is a wall to be pushed
                    // off sideways (`FightDecl::steepest`).
                    && (up.raw() > down.raw()
                        || (self.standable(index) && self.boardable(index))))
            {
                if step || up.raw() <= down.raw() {
                    p.y = sh.max.y;
                    landed = Some(index);
                } else {
                    p.y = sh.min.y.sub(body_height);
                }
            } else if px.abs().raw() <= pz.abs().raw() {
                p.x = p.x.add(px);
                shoved = true;
            } else {
                p.z = p.z.add(pz);
                shoved = true;
            }
            at = frame.local_to_world(p);
        }

        Contact {
            pos: at,
            landed,
            shoved,
        }
    }
}

// ---------------------------------------------------------------------------
// What it has out, and what is hitting it
// ---------------------------------------------------------------------------

impl Monster {
    /// The attack volume out this frame, in world space: a vertical cylinder's
    /// anchor, a radius and a height span.
    ///
    /// Exposed rather than kept private so the debug overlay draws *the volume
    /// the hit test uses*. An overlay that rebuilds the shape itself can drift
    /// from the rule it illustrates, and then it is confidently wrong at
    /// exactly the moment you are using it to work out why something missed.
    pub fn hit_volume(&self) -> Option<(V3, Fx, Fx, Fx)> {
        let Doing::Active { kind, left } = self.doing else {
            return None;
        };
        let m = self.sp().attack(kind);
        let decl = self.move_decl(kind);
        // A move whose species tests its own hit has no cylinder at all
        // (`MoveDecl::own_hit`): what it reaches and what is drawn for it are
        // the species' one shape.
        if (m.damage <= 0 && !decl.harmless) || decl.own_hit {
            return None;
        }
        // Its species' say on how big the move is now (`FightDecl::radius`).
        let m = match self.sp().fight.radius {
            Some(f) => Attack {
                hit_radius: f(self, kind, m.hit_radius),
                ..m
            },
            None => m,
        };
        // The spray's volume leaves the animal: so many metres a second along
        // the facing, from the first active frame. Everything else has zero
        // here and happens where it is standing.
        let flown = m
            .travel
            .mul(Fx::from_int(
                m.active.saturating_sub(left).saturating_sub(1) as i32,
            ))
            .mul(DT);
        // **A lobbed move lands at its aim point**, on the floor, wherever the
        // body is: the glob, the belly flop. Nothing about the bone it rides.
        // A lobbed move that travels **slides on from there** along the
        // facing -- the Pair's pounce skidding on, claws out -- which is zero
        // for every lobbed move that does not.
        if decl.lobbed {
            let at = self.aimed_at();
            let at = if m.travel.raw() != 0 {
                at.add(V3::from_turns(self.yaw).scale(flown))
            } else {
                at
            };
            // On the top its aim is on, for a species that says
            // (`FightDecl::lob_height`); on the floor otherwise.
            let up = match self.sp().fight.lob_height {
                Some(f) => f(self),
                None => Fx::ZERO,
            };
            return Some((
                V3::new(at.x, up, at.z),
                m.hit_radius,
                up.add(m.hit_low),
                up.add(m.hit_high),
            ));
        }
        let flown = if decl.stops_at_aim {
            flown.min(self.room_to_aim(m.hit_x))
        } else {
            flown
        };
        let rig = self.rig();
        // The anchor is authored in body space and carried by whichever bone
        // the move rides, so a bite whose head has been thrown forward puts its
        // hitbox where the head now is rather than where the head starts.
        //
        // Carried by the bone's motion **from rest**, not by its offset from
        // the hips. The first version added the whole offset, which put the
        // bite's volume nine to thirteen metres out on a move thrown at six,
        // and the sweep's five to eleven metres back on a tail that reaches
        // seven -- so the bite whiffed at its own ideal range and the tail root
        // was the one place behind the animal nothing could touch.
        let which = self.sp().follow_bone(m.follows);
        let carried = rig.bone[which].at.sub(rig.rest_at(which));
        let anchor = rig
            .to_world(V3::new(m.hit_x.add(flown), Fx::ZERO, m.hit_z))
            .add(V3::new(carried.x, Fx::ZERO, carried.z));
        let base = rig.origin.y;
        Some((
            anchor,
            m.hit_radius,
            base.add(m.hit_low),
            base.add(m.hit_high),
        ))
    }

    /// **What is coming, and where it will land.** The volume the move in
    /// progress will have when its hit comes out, for drawing on the floor
    /// through the windup -- and the volume it has, while it is out.
    ///
    /// Built by asking [`Monster::hit_volume`] about this animal posed on its
    /// first active frame, so it is the hit test's own answer rather than a
    /// second description of the move: the rule the debug overlay follows,
    /// applied to what a player sees. It moves with the windup, because the
    /// windup follows its target; on the first active frame it is exactly the
    /// hit. See `docs/design/monsters.md` §"Reading it".
    pub fn telegraph(&self) -> Option<Telegraph> {
        let (kind, live, progress, frames) = match self.doing {
            Doing::Startup { kind, left } => {
                let m = self.sp().attack(kind);
                (kind, false, through(left, m.startup), m.active)
            }
            Doing::Active { kind, left } => (kind, true, Fx::ONE, left),
            _ => return None,
        };
        let m = self.sp().attack(kind);
        let (anchor, radius, low, high) = if live {
            self.hit_volume()?
        } else {
            let mut landing = *self;
            landing.doing = Doing::Active {
                kind,
                left: m.active,
            };
            landing.hit_volume()?
        };
        let sweep = m
            .travel
            .add(m.advance)
            .mul(Fx::from_int(frames as i32))
            .mul(DT);
        // A lane that stops at the aim point is drawn stopping there: the line is
        // the hit test's own answer, solid in the way and all.
        let sweep = if self.move_decl(kind).stops_at_aim {
            let along = V3::from_turns(self.yaw);
            let gone = anchor.sub(self.pos).dot(along);
            let mark = self.aimed_at().sub(self.pos).dot(along);
            sweep.min(mark.sub(gone).max(Fx::ZERO))
        } else {
            sweep
        };
        Some(Telegraph {
            kind,
            anchor,
            radius,
            low,
            high,
            along: V3::from_turns(self.yaw),
            sweep,
            progress,
            live,
        })
    }

    /// One of its moves' declarations.
    pub fn move_decl(&self, kind: u8) -> crate::species::MoveDecl {
        let sp = self.sp();
        sp.moves[(kind as usize).min(sp.moves.len() - 1)]
    }

    /// The brain's aim point, on the floor. See [`Brain::aim`].
    pub fn aimed_at(&self) -> V3 {
        V3::new(
            crate::lore::from_cm(self.brain.aim[0]),
            Fx::ZERO,
            crate::lore::from_cm(self.brain.aim[1]),
        )
    }

    /// Set the brain's aim point. See [`Brain::aim`].
    pub fn aim_at(&mut self, at: V3) {
        self.brain.aim = [crate::lore::to_cm(at.x), crate::lore::to_cm(at.z)];
    }

    /// How far a travelling volume that starts `start` metres ahead of the
    /// body may go before it reaches the aim point, along the facing.
    fn room_to_aim(&self, start: Fx) -> Fx {
        let along = V3::from_turns(self.yaw);
        self.aimed_at()
            .sub(self.pos)
            .dot(along)
            .sub(start)
            .max(Fx::ZERO)
    }

    /// Does the volume out this frame reach a body standing at `world`?
    ///
    /// Height genuinely decides this one -- a stomp aimed at the ground does
    /// not reach someone standing on the creature's back -- which is the
    /// difference between a monster and a fighter, and the reason this is not
    /// the flat test `state::resolve_hit` uses.
    pub fn reaches(&self, world: V3, hurt_height: Fx, body_radius: Fx) -> bool {
        let Some((anchor, radius, low, high)) = self.hit_volume() else {
            return false;
        };
        let flat = V3::new(world.x.sub(anchor.x), Fx::ZERO, world.z.sub(anchor.z)).flat_len();
        if flat.raw() > radius.add(body_radius).raw() {
            return false;
        }
        world.y.add(hurt_height).raw() >= low.raw() && world.y.raw() <= high.raw()
    }

    /// The point of the creature's body nearest `world`.
    ///
    /// Each part's box, clamped to in the part's own frame and carried back
    /// out, and the one whose point is nearest **in three dimensions** wins.
    /// What asks is the Reaver's shadow deciding which way to face -- see
    /// `aim::shadow_faces` -- and it wants the flank it could reach, not the
    /// middle of an animal twelve metres long. Measured flat, the nearest
    /// thing to a shadow standing under the creature's neck was the head five
    /// metres overhead, and the copy turned to swing at the air beneath it.
    pub fn nearest_to(&self, world: V3) -> V3 {
        let rig = self.rig();
        let sp = self.sp();
        let mut best: Option<(V3, Fx)> = None;
        for index in 0..sp.parts.len() {
            if !rig.there(index) {
                continue;
            }
            let sh = sp.shape(index);
            let frame = rig.of(index);
            let local = frame.world_to_local(world);
            let clamped = V3::new(
                local.x.clamp(sh.min.x, sh.max.x),
                local.y.clamp(sh.min.y, sh.max.y),
                local.z.clamp(sh.min.z, sh.max.z),
            );
            let at = frame.local_to_world(clamped);
            let gap = crate::math::big_len(at.sub(world));
            if best.is_none_or(|(_, seen)| gap.raw() < seen.raw()) {
                best = Some((at, gap));
            }
        }
        best.map_or(self.pos, |(at, _)| at)
    }

    /// Which part an attack touches, or `None`.
    ///
    /// When it touches more than one, the **softest** wins: the ridge lies over
    /// the barrel and the nape over the neck, and a player who reached a weak
    /// point has hit the weak point. Rewarding the aim rather than the array
    /// order is the only version of this a player could predict.
    pub fn part_struck(&self, centre: V3, radius: Fx, body_height: Fx) -> Option<usize> {
        self.part_struck_on(&self.rig(), centre, radius, body_height)
    }

    /// [`Monster::part_struck`] on a rig already posed this frame: a swing
    /// asked down its length asks it half a dozen times, and posing the body
    /// for each is most of the cost.
    pub fn part_struck_on(
        &self,
        rig: &Rig,
        centre: V3,
        radius: Fx,
        body_height: Fx,
    ) -> Option<usize> {
        let sp = self.sp();
        let mut best: Option<(usize, Fx)> = None;
        for index in 0..sp.parts.len() {
            if !rig.there(index) {
                continue;
            }
            let sh = sp.shape(index);
            let frame = rig.of(index);
            let feet = frame.world_to_local(centre);
            let head = frame.world_to_local(V3::new(centre.x, centre.y.add(body_height), centre.z));
            let low = feet.y.min(head.y);
            let high = feet.y.max(head.y);
            if flat_gap(feet.x, feet.z, sh.min, sh.max).raw() > radius.raw() {
                continue;
            }
            if high.raw() < sh.min.y.raw() || low.raw() > sh.max.y.raw() {
                continue;
            }
            let soft = self.sp().vulnerability(index);
            if best.is_none_or(|(_, seen)| soft.raw() > seen.raw()) {
                best = Some((index, soft));
            }
        }
        best.map(|(i, _)| i)
    }

    /// Which part a *beam* passes through, and how far along it that happens.
    ///
    /// The Elementalist's auto is a ray rather than a volume at a point, so it
    /// asks a different question of the creature: not "which parts does this
    /// bubble overlap" but "which part does this line reach first". That also
    /// makes the softest-wins tiebreak in `part_struck` unnecessary here --
    /// that rule exists because a sphere sits inside several boxes at once and
    /// the player could not have chosen between them. A line can: the ridge is
    /// in front of the barrel from above and behind it from the side, and
    /// whichever the shot reaches first is the one that was aimed at.
    ///
    /// `swell` is the shot's own radius, added to the box.
    pub fn part_struck_along(
        &self,
        from: V3,
        dir: V3,
        limit: Fx,
        swell: Fx,
    ) -> Option<(usize, Fx)> {
        let rig = self.rig();
        let out = V3::new(swell, swell, swell);
        let sp = self.sp();
        let mut best: Option<(usize, Fx)> = None;
        for index in 0..sp.parts.len() {
            if !rig.there(index) {
                continue;
            }
            let sh = sp.shape(index);
            let frame = rig.of(index);
            let o = frame.world_to_local(from);
            let d = frame.rot.unapply(dir);
            let Some(dist) = crate::math::ray_hits_box(o, d, sh.min.sub(out), sh.max.add(out))
            else {
                continue;
            };
            if dist.raw() > limit.raw() {
                continue;
            }
            if best.is_none_or(|(_, seen)| dist.raw() < seen.raw()) {
                best = Some((index, dist));
            }
        }
        best
    }
}

// ---------------------------------------------------------------------------
// Damage, and the ladder of ways to be in trouble
// ---------------------------------------------------------------------------

impl Monster {
    /// How hurt it is, as a percentage of its pool.
    fn missing(&self) -> i32 {
        let max = self.sp().health().max(1);
        ((max - self.health).max(0) * 100) / max
    }

    /// A threshold, after the desperation discount.
    ///
    /// **Both thresholds fall as it is worn down.** That is the shape of the
    /// fight: at full health nothing short of a real burst moves it, and by the
    /// end an ordinary combo is enough. A hunt that starts methodical and ends
    /// frantic is the whole reason this is a curve rather than a constant.
    fn threshold(&self, base: i32) -> i32 {
        let cut = (self.strain_desperation() * self.missing()) / 100;
        (base * (100 - cut).clamp(5, 100)) / 100
    }

    /// Recent damage needed before crowd control means anything to it.
    pub fn cc_bar(&self) -> i32 {
        self.threshold(self.sp().cc_strain())
    }

    /// Recent damage needed to break it out of what it is doing.
    pub fn interrupt_bar(&self) -> i32 {
        self.threshold(self.sp().interrupt_strain())
    }

    /// Has it been hurt hard enough, recently enough, to be moved around?
    pub fn susceptible(&self) -> bool {
        self.strain >= self.cc_bar()
    }

    /// **A blow, asked of its guard first**: its species' guard
    /// (`FightDecl::guard`) says what the guard made of it, and a blow the
    /// guard turned deals nothing. A species without a guard takes every blow
    /// as [`Monster::take_hit`] always took it. Returns what went in, and
    /// what the guard said.
    pub fn take_blow(&mut self, part: usize, raw: i32, blow: &Blow) -> (i32, Guarded) {
        let guarded = match self.sp().fight.guard {
            Some(guard) if self.alive() => guard(self, part, blow),
            _ => Guarded::Lands,
        };
        if guarded.turned() {
            // Hit while it is still taking you in, and it has taken you in.
            self.brain.grace = 0;
            return (0, guarded);
        }
        (self.take_hit(part, raw), guarded)
    }

    /// **Would a blockable blow from `from` be stopped, now?** Its species'
    /// answer (`FightDecl::covers`), without changing anything: what the
    /// fight report's "guarded" window and the drawn guard read. False for a
    /// species with no guard.
    pub fn covers(&self, from: V3) -> bool {
        self.alive() && self.sp().fight.covers.is_some_and(|f| f(self, from))
    }

    /// Land a hit on a part. Returns the damage that actually went in, after
    /// the part's own vulnerability, which is what the caller should show.
    pub fn take_hit(&mut self, part: usize, raw: i32) -> i32 {
        if !self.alive() {
            return 0;
        }
        // Hit while it is still taking you in, and it has taken you in.
        self.brain.grace = 0;
        let worth = Fx::from_int(raw).mul(self.sp().vulnerability(part));
        // What its hide is worth on that part this frame, if its species
        // says: the Mireback's coat. Not asked at all otherwise, so a creature
        // without one takes exactly the arithmetic it always did.
        let worth = match self.sp().fight.hide {
            Some(hide) => worth.mul(hide(self, part)),
            None => worth,
        };
        let dealt = worth.to_int().max(0);
        self.health = (self.health - dealt).max(0);
        self.strain += dealt;
        if self.sp().is_weak_point(part) {
            self.poise += dealt;
        }

        if self.health <= 0 {
            self.doing = Doing::Dead;
            return dealt;
        }

        // Its species may decide what the hit did instead of the ladder below.
        if let Some(struck) = self.sp().fight.struck {
            if struck(self, part, dealt) {
                return dealt;
            }
        }

        // A foot going is the ground game's whole payout, so it is checked
        // before anything else can claim the frame.
        if let Some(slot) = self.sp().break_slot(part).filter(|s| self.breaks[*s] > 0) {
            self.breaks[slot] -= dealt;
            if self.breaks[slot] <= 0 {
                self.breaks[slot] = 0;
                self.break_a_leg(part);
                return dealt;
            }
        }

        if self.poise >= self.sp().poise_max() && !matches!(self.doing, Doing::Toppled { .. }) {
            // Over it goes. This is the window the climb is for, so it resets
            // the pool rather than draining it: you earn the next one again.
            //
            // Unlike a flinch, this **does** interrupt a live hitbox. Legs
            // going out from under it mid-bite is a different event from a
            // flinch, and one the player has spent a whole climb earning.
            self.poise = 0;
            self.topple();
            return dealt;
        }

        // **The interrupt.** Enough damage in a short enough window breaks it
        // out of whatever it is doing -- and unlike a flinch, out of a live
        // hitbox too. That is the point: a burst big enough to stop a charge is
        // a decision worth building a kit around, and the cost is that the
        // strain is spent and has to be earned again.
        if self.strain >= self.interrupt_bar() && self.doing.attacking().is_some() {
            self.strain = 0;
            self.doing = Doing::Flinch {
                left: self.sp().flinch_frames(),
            };
            return dealt;
        }

        // A flinch interrupts, but never an active frame: a creature whose hit
        // could be cancelled by being hit is a creature you never have to
        // trade with, and trading is most of what the ground game is.
        let interruptible = !matches!(
            self.doing,
            Doing::Active { .. } | Doing::Toppled { .. } | Doing::Stumble { .. }
        );
        if dealt >= self.sp().flinch_threshold() && interruptible {
            self.doing = Doing::Flinch {
                left: self.sp().flinch_frames(),
            };
        }
        dealt
    }

    /// A foot has gone. Down it comes.
    ///
    /// The stumble is the moment; `lame_layer` is the rest of the fight. Both
    /// matter and they are different things: the moment is the mount window,
    /// and the limp is what makes the animal easier to stay ahead of
    /// afterwards.
    fn break_a_leg(&mut self, part: usize) {
        let front = self
            .sp()
            .legs
            .iter()
            .find(|leg| leg.foot == part)
            .is_some_and(|leg| leg.front);
        self.doing = Doing::Stumble {
            left: self.sp().stumble_frames(),
            front,
        };
        self.speed = Fx::ZERO;
        self.yaw_rate = Fx::ZERO;
        self.strain = 0;
    }

    fn topple(&mut self) {
        self.doing = Doing::Toppled {
            left: self.sp().topple_frames(),
        };
        self.speed = Fx::ZERO;
        self.yaw_rate = Fx::ZERO;
        self.strain = 0;
    }

    /// Offer the creature a piece of crowd control and see what it takes.
    ///
    /// Nothing, unless it is [`susceptible`](Monster::susceptible) -- and then a
    /// weakened version: a knock-up takes it off balance rather than off the
    /// ground, a grab roots it rather than dragging it, and a slow lands at a
    /// fraction of its strength. **Partial on purpose.** A monster that took
    /// full crowd control would be a monster you lock down, and the point of
    /// putting a threshold in front of it is that the *same button* means
    /// something in both halves of the game without meaning the same thing.
    ///
    /// The strain is spent, so control has to be bought again rather than held.
    pub fn take_control(&mut self, cc: Control) -> Took {
        let mut took = Took::default();
        if !self.alive() || !cc.is_anything() || !self.susceptible() {
            return took;
        }
        self.strain = (self.strain - self.cc_bar()).max(0);

        if cc.slow_frames > 0 {
            // Softened toward "no slow at all" by whatever fraction the Oven
            // says a creature this size is willing to feel.
            let bite = self.sp().cc_slow_bite();
            let mul = Fx::ONE.sub(Fx::ONE.sub(cc.slow_mul).mul(bite));
            self.slow(cc.slow_frames, mul);
            took.slowed = true;
        }
        if cc.grabs > 0 {
            self.rooted = self.rooted.max(
                Fx::from_int(cc.grabs as i32)
                    .mul(self.sp().cc_root())
                    .to_int() as u16,
            );
            self.speed = Fx::ZERO;
            took.rooted = true;
        }
        if cc.launch.raw() > 0 && !matches!(self.doing, Doing::Toppled { .. }) {
            // A knock-up on something this heavy is a trip, not a lift. Its
            // length scales with the launch the move would have given a
            // fighter, so the moves that were built to open somebody up are the
            // ones that open this up too.
            let frames = self
                .sp()
                .cc_stumble()
                .mul(cc.launch)
                .to_int()
                .clamp(1, self.sp().stumble_frames() as i32) as u16;
            self.doing = Doing::Stumble {
                left: frames,
                front: true,
            };
            self.speed = Fx::ZERO;
            self.yaw_rate = Fx::ZERO;
            took.stumbled = true;
        }
        took
    }

    /// Take a slow. The strongest one on you is the one that counts, the same
    /// rule the fighters follow.
    pub fn slow(&mut self, frames: u16, mul: Fx) {
        if self.slowed == 0 || mul.raw() < self.slow_mul.raw() {
            self.slow_mul = mul;
        }
        self.slowed = self.slowed.max(frames);
    }
}

// ---------------------------------------------------------------------------
// The control algorithm
// ---------------------------------------------------------------------------

impl Monster {
    /// Xorshift32. Deterministic, seeded from the snapshot, advanced only from
    /// inside the tick -- so a rollback re-rolls the same numbers.
    fn roll(&mut self) -> u32 {
        let mut x = self.brain.rng;
        if x == 0 {
            x = 0x2545_F491;
        }
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.brain.rng = x;
        x
    }

    /// Take a look, if one is due.
    ///
    /// Between glances the creature is working from stale information. That is
    /// the whole difficulty model: it cannot react to a button because it is
    /// not looking, and a player who changes direction between two glances has
    /// done something real.
    fn glance(&mut self, quarry: &[Quarry], senses: &Senses, lore: &crate::lore::Lore) {
        // **Eyes that are not a glance** (`FightDecl::sight`): a species that
        // keeps its own record of what it saw -- the Mantis's delay line --
        // takes its sample from there, every frame, and never from the
        // present.
        if let Some(sight) = self.sp().fight.sight {
            if let Some(s) = sight(self, lore) {
                self.brain.target = s.target;
                self.brain.seen = s.pos;
                self.brain.seen_vel = s.vel;
                self.brain.seen_stunned = s.stunned;
            }
            return;
        }
        if self.brain.glance_left > 0 {
            self.brain.glance_left -= 1;
            return;
        }
        self.brain.glance_left = self.glance_frames();
        // **What it cannot perceive it does not sample** (bestiary P5). A
        // fighter its species' filter rules out this glance is not there to
        // it; if nobody is, it takes its sample from what it heard, and if it
        // heard nothing it keeps the old sample -- which it goes on leading.
        let perceived = |i: usize| senses.perceived.get(i).copied().unwrap_or(true);

        // It looks at whoever is nearest and *not* on its back: there is
        // nothing to bite at up there, and if a second hunter is on the ground
        // they are the one worth facing. Only when everyone is aboard does it
        // fall back to a rider, at which point every attack scores badly and
        // the bucking moves win by default -- which is the behaviour we wanted
        // and did not have to write down twice.
        let nearest = |aboard_counts: bool| -> Option<usize> {
            let mut best: Option<(usize, Fx)> = None;
            for (i, q) in quarry.iter().enumerate() {
                if !q.alive || (!aboard_counts && q.aboard) || !perceived(i) {
                    continue;
                }
                let d = crate::math::wide_flat_len(q.pos.sub(self.pos));
                if best.is_none_or(|(_, seen)| d.raw() < seen.raw()) {
                    best = Some((i, d));
                }
            }
            best.map(|(i, _)| i)
        };
        let Some(mut pick) = nearest(false).or_else(|| nearest(true)) else {
            if let Some(h) = senses.heard {
                self.brain.seen = h.at;
                self.brain.seen_vel = V3::ZERO;
                self.brain.seen_stunned = false;
                if (h.who as usize) < quarry.len() {
                    self.brain.target = h.who;
                }
            }
            return;
        };
        // **It keeps the target it has.** Nearest-wins alone made it turn
        // away from the fighter it was trading with the moment their partner
        // -- or an idle second fighter -- drifted a metre closer, which from
        // the first fighter's side reads as an animal losing interest and
        // lumbering off. Somebody else has to be *much* nearer, or the one it
        // has must be gone or aboard, before its attention moves.
        let held = self.brain.target as usize;
        if held < quarry.len() && held != pick {
            let q = quarry[held];
            let usable = q.alive && (!q.aboard || quarry[pick].aboard) && perceived(held);
            let mine = crate::math::wide_flat_len(q.pos.sub(self.pos));
            let theirs = crate::math::wide_flat_len(quarry[pick].pos.sub(self.pos));
            if usable && theirs.raw() >= mine.mul(self.sp().target_switch()).raw() {
                pick = held;
            }
        }
        self.brain.target = pick as u8;
        self.brain.seen = quarry[pick].pos;
        self.brain.seen_vel = quarry[pick].vel;
        self.brain.seen_stunned = quarry[pick].stunned;
    }

    /// Where it thinks the target will be in `frames` frames.
    ///
    /// The horizon is the move's own startup, which is the physically right
    /// scaling and costs one knob instead of one per move: a lunge that takes
    /// half a second leads half a second's worth. `lead` is how much of that
    /// extrapolation it actually trusts -- at zero it swipes at where you were.
    pub fn lead_point(&self, frames: u16) -> V3 {
        let horizon = Fx::from_int(frames as i32).mul(DT).mul(self.lead());
        self.brain.seen.add(self.brain.seen_vel.scale(horizon))
    }

    /// How much the creature wants to throw a given move right now.
    ///
    /// Range and angle multiply rather than add, because a move with perfect
    /// spacing and the target behind it is not half a good idea.
    pub fn appetite(&self, kind: u8, riders: i32) -> i32 {
        if self.brain.cooldown[(kind as usize).min(self.sp().moves.len() - 1)] > 0 {
            return 0;
        }
        let m = self.sp().attack(kind);
        let aim = self.lead_point(m.startup);
        let to = V3::new(aim.x.sub(self.pos.x), Fx::ZERO, aim.z.sub(self.pos.z));
        let range = to.flat_len();
        let facing = V3::from_turns(self.yaw);
        let cos = facing.dot(to.normalized());

        let range_fit = Fx::ONE
            .sub(range.sub(m.ideal_range).abs().div(m.range_span))
            .clamp(Fx::ZERO, Fx::ONE);
        let aim_fit = Fx::ONE
            .sub(cos.sub(m.aim_cos).abs().div(m.aim_span))
            .clamp(Fx::ZERO, Fx::ONE);

        let fit = range_fit.mul(aim_fit);
        let mut score = Fx::from_int(m.weight).mul(fit).to_int();
        score += riders * m.rider_weight;

        // **Walking up to it is provoked.** A target closing on it faster than
        // a stroll raises its appetite for whatever forward move fits, so the
        // approach is met by something rather than watched. The bonus is
        // scaled by the fit, so a move that does not reach the approach does
        // not score for it.
        let toward = to.normalized().scale(Fx::ONE.neg());
        let closing = self.brain.seen_vel.dot(toward);
        if closing.raw() > self.sp().closing_speed().raw() && m.aim_cos.raw() > 0 && m.damage > 0 {
            score += Fx::from_int(self.sp().closing_appetite()).mul(fit).to_int();
        }
        // **A stunned target is a target to follow up on.** The sweep's
        // stagger and the spray's root are only worth having if the animal
        // presses them, and the bite and the charge are what it presses with.
        if self.brain.seen_stunned && m.aim_cos.raw() > 0 && m.damage > 0 {
            score += Fx::from_int(self.sp().combo_appetite()).mul(fit).to_int();
        }

        // Wounded animals commit harder, and they commit to bigger swings.
        score += (self.sp().hurt_aggression() * self.missing() * m.damage) / 10_000;

        if self.brain.last_move == kind && self.brain.repeat_left > 0 {
            let span = self.sp().variety_frames().max(1) as i32;
            score -= (self.sp().variety_penalty() * self.brain.repeat_left as i32) / span;
        }
        score
    }

    /// [`Monster::appetite`], with its species' own terms after it
    /// (`FightDecl::appetite`) and nothing at all for a move the brain never
    /// picks. What the choice is made on.
    pub fn appetite_in(&self, kind: u8, riders: i32, mind: &Mind) -> i32 {
        let sp = self.sp();
        if self.move_decl(kind).never_chosen {
            return 0;
        }
        let base = self.appetite(kind, riders);
        match sp.fight.appetite {
            Some(hook) if self.brain.cooldown[(kind as usize).min(sp.moves.len() - 1)] == 0 => {
                hook(self, kind, base, mind)
            }
            _ => base,
        }
    }

    /// Pick something to do, if anything is worth doing.
    ///
    /// Not the maximum. Everything scoring at least `decisiveness` of the best
    /// goes into a weighted draw: at full decisiveness it always throws the
    /// best move and is therefore a script you can memorise, and at zero it is
    /// noise. In between, the *distribution* is learnable while the next move
    /// is not, which is the only version of "hard but fair" that survives a
    /// player who has fought it fifty times.
    fn choose(&mut self, riders: i32, mind: &Mind) {
        let sp = self.sp();
        let mut scores = [0i32; MAX_MOVES];
        let scores = &mut scores[..sp.moves.len()];
        let mut best = 0;
        for (kind, slot) in scores.iter_mut().enumerate() {
            *slot = self.appetite_in(kind as u8, riders, mind);
            best = best.max(*slot);
        }
        if best <= 0 {
            return;
        }
        let cut = ((best as i64 * self.decisiveness() as i64) / 100) as i32;
        let total: i32 = scores.iter().filter(|s| **s >= cut && **s > 0).sum();
        if total <= 0 {
            return;
        }

        let mut ticket = (self.roll() % total as u32) as i32;
        let mut chosen = None;
        for (kind, score) in scores.iter().enumerate() {
            if *score < cut || *score <= 0 {
                continue;
            }
            ticket -= *score;
            if ticket < 0 {
                chosen = Some(kind as u8);
                break;
            }
        }
        let kind = chosen.unwrap_or(0);
        self.begin(kind, mind);
    }

    /// **Start a move**: everything the brain does as a move commits, whether
    /// it chose the move or the move was declared to follow another
    /// (`MoveDecl::then`).
    fn begin(&mut self, kind: u8, mind: &Mind) {
        let sp = self.sp();
        let m = sp.attack(kind);
        // A move that mirrors goes to whichever side the target is on -- the
        // Ridgeback's sweep. Decided once, here, and held for the move: the
        // yaw is locked for the same reason, and a tail that changed its mind
        // mid-whip would be a telegraph that lied.
        self.brain.mirror = sp.moves[kind as usize].mirrors_to_target_side && {
            let right = V3::from_turns(self.yaw.add(crate::math::QUARTER_TURN));
            let to = self.brain.seen.sub(self.pos);
            right.dot(to).raw() < 0
        };
        self.doing = Doing::Startup {
            kind,
            left: m.startup,
        };
        self.hit_used = false;
        self.brain.last_move = kind;
        self.brain.repeat_left = self.sp().variety_frames();
        self.brain.cooldown[kind as usize] = m.cooldown;
        self.brain.think_left = 0;
        if sp.moves[kind as usize].lobbed {
            self.lob(kind);
        }
        if let Some(commit) = sp.fight.commit {
            commit(self, kind, mind);
        }
    }

    /// **Aim a lobbed move**: where the target will be when it lands, kept
    /// inside the move's own range -- the aim point, held from here on. What
    /// the brain does as a lobbed move commits; a species may aim it again
    /// (`FightDecl::commit`).
    pub fn lob(&mut self, kind: u8) {
        let m = self.sp().attack(kind);
        let aim = self.lead_point(m.startup);
        let to = V3::new(aim.x.sub(self.pos.x), Fx::ZERO, aim.z.sub(self.pos.z));
        let far = crate::math::wide_flat_len(to);
        let near = m.ideal_range.sub(m.range_span).max(Fx::ZERO);
        let most = m.ideal_range.add(m.range_span);
        let reach = far.clamp(near, most);
        let dir = if far.raw() > 0 {
            crate::math::wide_normalized(to)
        } else {
            V3::from_turns(self.yaw)
        };
        self.aim_at(self.pos.add(dir.scale(reach)));
    }

    /// Turn. A proportional controller behind a rate limit and an acceleration
    /// limit.
    ///
    /// The rate limit is how fast it can turn. The **acceleration** limit is
    /// what gives it mass: it cannot reverse a swing instantly, so it overshoots
    /// when it has been turning hard, and that overshoot is the window a player
    /// gets for cutting back across its nose. A rate limit alone would not give
    /// them one.
    fn steer(&mut self, prowl: Option<V3>) {
        // **The windup follows you; the hit does not.** During a startup the
        // animal keeps turning toward where it thinks you will be, at a
        // fraction of its free turn rate, and the yaw locks on the first active
        // frame. Walking out of the way of a tell is therefore not an answer
        // -- a timed dodge, a jump, or a real change of direction is -- while
        // the hit itself still commits, so whiff punishment exists. The
        // fraction is `startup_tracking`; at zero this is the old rule.
        //
        // **Only a move aimed ahead of it tracks.** Turning toward the target
        // during a sweep's windup swings the tail *away* from them, which is a
        // tell that makes its own move miss.
        let winding = matches!(self.doing, Doing::Startup { kind, .. } if self.sp().attack(kind).aim_cos.raw() > 0);
        if !self.doing.free() && !winding {
            // Committed, or down. Whatever swing was in progress bleeds off
            // rather than stopping dead, so a move does not visibly clamp the
            // animal mid-turn.
            self.yaw_rate = self.yaw_rate.mul(self.sp().turn_settle());
            self.yaw = self.yaw.add(self.yaw_rate.mul(DT));
            return;
        }
        // Prowling, it leads by its walking horizon. Winding up, it leads to
        // **where you will be when the hit arrives**: the startup it has left,
        // and for a move that travels -- the charge, the spray, the bite's
        // lunge -- the time the hit takes to cross the gap. Leading by the
        // startup alone aimed a charge at where a sidestepping fighter would
        // be when it set off, and they had walked out of it by the time it
        // got there.
        let horizon = match self.doing {
            Doing::Startup { kind, left } if winding => {
                let m = self.sp().attack(kind);
                let speed = m.travel.add(m.advance);
                let gap = self
                    .brain
                    .seen
                    .sub(self.pos)
                    .flat_len()
                    .sub(m.hit_x.abs())
                    .max(Fx::ZERO);
                let crossing = if speed.raw() > 0 {
                    gap.div(speed.mul(DT)).to_int().clamp(0, m.active as i32) as u16
                } else {
                    0
                };
                left.saturating_add(crossing)
            }
            _ => self.sp().prowl_lead(),
        };
        // Walking somewhere of its own, it faces where it is going -- until it
        // is within a stride of it, and then it faces you.
        let arrived = |to: V3| {
            crate::math::wide_flat_dist(to, self.pos).raw() <= self.sp().gait_stride().raw()
        };
        let aim = match prowl {
            Some(to) if self.doing.free() && !arrived(to) => to,
            _ => self.lead_point(horizon),
        };
        let want = crate::math::atan2_turns(aim.z.sub(self.pos.z), aim.x.sub(self.pos.x));
        let error = crate::math::wrap_turns(want.sub(self.yaw));

        // Turning toward a side is driven off the legs on that side, so
        // breaking one is something the player can see the consequence of.
        // Increasing yaw swings toward positive Z, which is the creature's own
        // right.
        let toward_right = error.raw() > 0;
        let lame = self
            .sp()
            .legs
            .iter()
            .filter(|leg| (leg.side > 0) == toward_right)
            .filter(|leg| self.broken(leg.foot))
            .count() as i32;
        let mut cap = self.sp().turn_rate_max();
        for _ in 0..lame {
            cap = cap.mul(self.sp().turn_hurt());
        }
        if self.slowed > 0 {
            cap = cap.mul(self.slow_mul);
        }
        if winding {
            cap = cap.mul(self.sp().startup_tracking());
        }

        let desired = error.mul(self.sp().turn_gain()).clamp(cap.neg(), cap);
        let budget = self.sp().turn_accel().mul(DT);
        let change = desired.sub(self.yaw_rate).clamp(budget.neg(), budget);
        self.yaw_rate = self.yaw_rate.add(change);
        self.yaw = self.yaw.add(self.yaw_rate.mul(DT));
    }

    /// How much of its speed it still has: slows and broken legs both cost it.
    fn hobbled(&self) -> Fx {
        let (front, rear) = self.lameness();
        let mut mul = if self.slowed > 0 {
            self.slow_mul
        } else {
            Fx::ONE
        };
        for _ in 0..(front + rear) {
            mul = mul.mul(self.sp().leg_speed_hurt());
        }
        // And its species' say on its pace this frame (`FightDecl::pace`):
        // an enraged cat runs faster than its gallop.
        match self.sp().fight.pace {
            Some(f) => mul.mul(f(self)),
            None => mul,
        }
    }

    /// Walk. Forward along its own facing, and never sideways -- a quadruped
    /// that could strafe would make its turn limit decorative.
    fn walk(&mut self, arena: &Terrain, prowl: Option<V3>) {
        let bounds = &arena.arena.bounds;
        let want = match self.doing {
            _ if self.rooted > 0 => Fx::ZERO,
            // Noticing you: it stands its ground and turns to face you.
            Doing::Prowl if self.brain.grace > 0 => Fx::ZERO,
            Doing::Active { kind, .. } => self.sp().attack(kind).advance,
            Doing::Prowl => {
                let goal = prowl.unwrap_or(self.brain.seen);
                let to = V3::new(goal.x.sub(self.pos.x), Fx::ZERO, goal.z.sub(self.pos.z));
                let range = crate::math::wide_flat_len(to);
                // **It gallops when you run.** The clamp used to be the walk,
                // which is slower than a fighter's, so the whole fight was
                // walking away from it at leisure. Wanting to close a long gap
                // now runs it up to the gallop, and the gait blends to match;
                // the walk is what it does inside striking distance.
                //
                // **And it matches a target that is leaving.** The proportional
                // term alone asks for a walk at eight metres and a stroll at
                // seven, so a fighter backpedalling at their own walk from
                // just outside its reach was never caught: it only galloped
                // once they were far away, and slowed as it arrived. The speed
                // the target is moving away at is added, times `pursuit_gain`,
                // so a fleeing fighter is a galloped-at fighter whatever the
                // distance.
                let fleeing = self
                    .brain
                    .seen_vel
                    .dot(crate::math::wide_normalized(to))
                    .max(Fx::ZERO)
                    .mul(self.sp().pursuit_gain());
                // Somewhere of its own is somewhere to stand on; the
                // preferred distance is from a target.
                let stand_off = if prowl.is_some() {
                    Fx::ZERO
                } else {
                    self.sp().prowl_range()
                };
                let want = range
                    .sub(stand_off)
                    .mul(self.sp().approach_gain())
                    .add(fleeing)
                    .clamp(self.sp().back().neg(), self.sp().gallop());
                // **It turns before it runs.** Forward speed is scaled by how
                // squarely it is facing the target, so a creature that has
                // been got behind comes about on the spot rather than
                // galloping off in the wrong direction and swinging round in
                // an arc that ends beside you. Backing off is not scaled: that
                // is what it does when you are too close, whichever way it is
                // pointed.
                if want.raw() > 0 {
                    let ahead = V3::from_turns(self.yaw)
                        .dot(crate::math::wide_normalized(to))
                        .max(Fx::ZERO);
                    want.mul(ahead)
                } else {
                    want
                }
            }
            _ => Fx::ZERO,
        }
        .mul(self.hobbled());
        // **It pulls up short of the wall.** A charge covers twenty metres and
        // the arena is twenty-eight across, so a charge thrown from anywhere
        // but the middle ran into the edge, where the clamp below stopped it
        // dead in a frame -- an unbounded deceleration, which the grip test
        // reads as a buck, so it threw braced riders off the barrel. Within
        // its braking distance of the wall it brakes instead: speed squared
        // over twice the braking rate.
        let room = self.room_ahead(bounds);
        let stopping = self
            .speed
            .mul(self.speed)
            .div(self.sp().brake().mul(Fx::from_int(2)));
        let want = if self.speed.raw() > 0 && room.raw() <= stopping.raw() {
            Fx::ZERO
        } else {
            want
        };
        let walled = self.speed.raw() > 0 && room.raw() <= stopping.raw();
        let planted = walled || !matches!(self.doing, Doing::Prowl | Doing::Active { .. });
        let budget = if planted {
            self.sp().brake()
        } else {
            self.sp().accel()
        }
        .mul(DT);
        self.speed = match self.doing {
            // **A move that travels is launched, not accelerated into.** At
            // the walking acceleration a thirty-metre-a-second charge reached
            // about four metres a second by the end of its active window and
            // covered a metre and a half -- which is why the charge had never
            // once landed. It reaches its speed in a few frames now; the skid
            // after it is braking. See `tuning::monster_launch`.
            Doing::Active { kind, .. } if self.sp().attack(kind).advance.raw() > 0 && !walled => {
                let burst = self.sp().launch().mul(DT);
                self.speed
                    .add(want.sub(self.speed).clamp(burst.neg(), burst))
            }
            _ => self
                .speed
                .add(want.sub(self.speed).clamp(budget.neg(), budget)),
        };

        let forward = V3::from_turns(self.yaw);
        self.pos = self.pos.add(forward.scale(self.speed.mul(DT)));

        // Where it is in its stride. An accumulator rather than a ratio, for
        // the reason the fighters' is: a stride is longer at a gallop than at a
        // walk, so `distance / stride` jumps by whole cycles the moment the
        // gait changes, which reads as four legs teleporting at once.
        let covered = self.speed.abs().mul(DT).div(self.sp().gait_stride());
        self.stride = self
            .stride
            .wrapping_add(covered.raw().clamp(0, 65535) as u16);
        self.beat = self.beat.wrapping_add(self.sp().breath_rate());

        // It stays inside the arena and on the floor. Nothing in the
        // Ridgeback's move set takes it off the ground, and a creature that
        // size on a platform would be a camera problem rather than a fight. A
        // species whose height is its own (`FightDecl::keeps_height`: a cat on
        // a wall) is left where its hooks put it.
        let margin = self.sp().margin();
        self.pos.x = self
            .pos
            .x
            .clamp(bounds.lo_x.add(margin), bounds.hi_x.sub(margin));
        self.pos.z = self
            .pos
            .z
            .clamp(bounds.lo_z.add(margin), bounds.hi_z.sub(margin));
        if !self.sp().fight.keeps_height {
            // On the floor, wherever the floor is: a hill's crown on a hill
            // (`arena::relief`). Not on a solid -- the creature does not
            // collide with those (review.md `MACH-3`).
            self.pos.y = arena.relief_at(self.pos.x, self.pos.z);
        }
    }

    /// Metres of floor ahead of it before the wall it is kept off, along its
    /// facing.
    fn room_ahead(&self, bounds: &Bounds) -> Fx {
        let margin = self.sp().margin();
        let forward = V3::from_turns(self.yaw);
        let along = |pos: Fx, dir: Fx, lo: Fx, hi: Fx| -> Fx {
            if dir.raw() > 0 {
                hi.sub(margin).sub(pos).div(dir)
            } else if dir.raw() < 0 {
                lo.add(margin).sub(pos).div(dir)
            } else {
                Fx::MAX
            }
        };
        along(self.pos.x, forward.x, bounds.lo_x, bounds.hi_x)
            .min(along(self.pos.z, forward.z, bounds.lo_z, bounds.hi_z))
            .max(Fx::ZERO)
    }

    /// Advance the frame-data clock, exactly the way a fighter's does.
    fn tick_action(&mut self) {
        let rest = |m: &mut Monster| {
            m.brain.think_left = m.sp().think_frames();
            Doing::Prowl
        };
        self.doing = match self.doing {
            Doing::Dead => Doing::Dead,
            Doing::Prowl => Doing::Prowl,
            Doing::Startup { kind, left } if left > 0 => Doing::Startup {
                kind,
                left: left - 1,
            },
            Doing::Startup { kind, .. } => {
                self.hit_used = false;
                Doing::Active {
                    kind,
                    left: self.sp().attack(kind).active,
                }
            }
            Doing::Active { kind, left } if left > 0 => Doing::Active {
                kind,
                left: left - 1,
            },
            Doing::Active { kind, .. } => Doing::Recovery {
                kind,
                left: self.sp().attack(kind).recovery,
            },
            Doing::Recovery { kind, left } if left > 0 => Doing::Recovery {
                kind,
                left: left - 1,
            },
            Doing::Recovery { kind, .. } => {
                let next = rest(self);
                // Turning to face somebody it has just staggered would waste
                // the stagger: the sweep is a set-up, and a set-up is pressed.
                if self.sp().attack(kind).aim_cos.raw() < 0 && !self.brain.seen_stunned {
                    self.brain.think_left = self.sp().rear_pause();
                }
                next
            }
            Doing::Flinch { left } if left > 0 => Doing::Flinch { left: left - 1 },
            Doing::Flinch { .. } => rest(self),
            Doing::Stumble { left, front } if left > 0 => Doing::Stumble {
                left: left - 1,
                front,
            },
            Doing::Stumble { .. } => rest(self),
            Doing::Toppled { left } if left > 0 => Doing::Toppled { left: left - 1 },
            Doing::Toppled { .. } => rest(self),
        };
    }

    /// One tick of creature, in the proving ground. For the tests and tools
    /// that hold a creature without a world; the world calls [`Monster::step_in`].
    pub fn step(&mut self, quarry: &[Quarry]) {
        self.step_in(
            quarry,
            &Senses::ALL,
            &Terrain::bare(&crate::arena::proving_ground::ARENA),
        );
    }

    /// One tick of creature, on the ground as it stands: its bounds are the
    /// walls it is kept inside and pulls up short of, and -- for a species
    /// that `collides` -- its solids are what it walks into. `senses` is what
    /// its perception filter allows the glance this frame.
    pub fn step_in(&mut self, quarry: &[Quarry], senses: &Senses, arena: &Terrain) {
        self.step_with(quarry, senses, arena, &crate::lore::Lore::NONE);
    }

    /// [`Monster::step_in`], with the hunt's lore for its species' brain
    /// hooks to read (`FightDecl::appetite`, `prowl_to`, `commit`). What the
    /// world calls.
    pub fn step_with(
        &mut self,
        quarry: &[Quarry],
        senses: &Senses,
        arena: &Terrain,
        lore: &crate::lore::Lore,
    ) {
        if self.health <= 0 {
            self.doing = Doing::Dead;
            self.speed = Fx::ZERO;
            self.yaw_rate = Fx::ZERO;
            return;
        }
        self.glance(quarry, senses, lore);
        // **A declared chain** (`MoveDecl::then`): a move whose recovery ends
        // this frame and that always leads to another starts that one at
        // once, with no pause to think and no choice. Nothing in the cast
        // declared one before the Broodmother's screech.
        let chained = match self.doing {
            Doing::Recovery { kind, left: 0 } => self.move_decl(kind).then,
            _ => None,
        };
        self.tick_action();
        self.bleed_off();

        let riders = quarry.iter().filter(|q| q.alive && q.aboard).count() as i32;
        let noticing = self.brain.grace > 0;
        self.brain.grace = self.brain.grace.saturating_sub(1);
        let mind = Mind {
            quarry,
            ground: arena,
            lore,
        };
        if let Some(next) = chained.filter(|_| self.doing.free()) {
            self.begin(next, &mind);
        }
        if self.doing.free() && !noticing {
            if self.brain.think_left > 0 {
                self.brain.think_left -= 1;
            } else {
                self.choose(riders, &mind);
            }
            // Poise comes back while it is on its feet, so a topple has to be
            // earned again rather than saved up across a whole fight.
            self.poise = (self.poise - self.sp().poise_regen()).max(0);
        }
        self.brain.repeat_left = self.brain.repeat_left.saturating_sub(1);

        let prowl = match self.sp().fight.prowl_to {
            Some(hook) if self.doing.free() => hook(self, &mind),
            _ => None,
        };
        self.steer(prowl);
        self.walk(arena, prowl);
        self.fence(arena);
    }

    /// **Walk into the arena's solids**, for a species that `collides`: pushed
    /// out sideways from anything taller than its row's `StepOver`, at its
    /// row's `BodyRadius`, and told so through its `bumped` hook (the
    /// Hornback's charge into a rock). A species that does not collide is only
    /// kept inside the bounds, which is what the Ridgeback has always had.
    fn fence(&mut self, arena: &Terrain) {
        let sp = self.sp();
        if !sp.fight.collides {
            return;
        }
        use crate::species::FightField;
        let radius = sp.fight_fx(FightField::BodyRadius);
        let step = sp.fight_fx(FightField::StepOver);
        let (at, push) = arena.fence(self.pos, radius, step);
        if push != V3::ZERO {
            self.pos = at;
            // Whatever of its speed went into the solid is gone.
            let forward = V3::from_turns(self.yaw);
            let into = forward.dot(crate::math::wide_normalized(push));
            if into.raw() < 0 {
                self.speed = self.speed.mul(Fx::ONE.add(into)).max(Fx::ZERO);
            }
            if let Some(bumped) = sp.fight.bumped {
                bumped(self, push);
            }
        }
    }

    /// **Where it sees from**: its head -- the last bone of its neck -- or its
    /// root bone for a species with no neck. What a perception filter measures
    /// sight, feel and hearing from.
    pub fn head(&self) -> V3 {
        let sp = self.sp();
        if !sp.has_body() {
            return self.pos;
        }
        let bone = sp.neck.last().copied().unwrap_or(0);
        self.rig().bone[bone.min(sp.bones.len() - 1)].at
    }

    /// **Damage from the floor**: a burning hazard under it. Health and strain,
    /// as any blow; no part, so no hide multiplier, no break and no poise --
    /// fire under a body is not a blow on one of its parts. Returns what went
    /// in.
    pub fn scorch(&mut self, damage: i32) -> i32 {
        if !self.alive() || damage <= 0 {
            return 0;
        }
        self.brain.grace = 0;
        let dealt = damage.min(self.health);
        self.health -= dealt;
        self.strain += dealt;
        if self.health <= 0 {
            self.doing = Doing::Dead;
        }
        dealt
    }

    /// Frames until it can start another move, as things stand.
    ///
    /// Zero while a move is winding up or out; the frames left plus its pause
    /// while it is recovering or down; the pause alone while it prowls. This
    /// is the one number every opening is made of, and the fight report
    /// divides the fight by it: a window shorter than a reaction and a swing
    /// is no window, and one long enough to walk in on is a free one.
    pub fn frames_until_free(&self) -> u16 {
        let pause = |kind: Option<u8>| match kind {
            Some(k) if self.sp().attack(k).aim_cos.raw() < 0 && !self.brain.seen_stunned => {
                self.sp().rear_pause()
            }
            _ => self.sp().think_frames(),
        };
        match self.doing {
            Doing::Dead => u16::MAX,
            Doing::Startup { .. } | Doing::Active { .. } => 0,
            Doing::Prowl => self.brain.think_left,
            Doing::Recovery { kind, left } => left.saturating_add(pause(Some(kind))),
            Doing::Flinch { left } | Doing::Stumble { left, .. } | Doing::Toppled { left } => {
                left.saturating_add(pause(None))
            }
        }
    }

    /// Everything that ticks down on its own.
    ///
    /// The strain decays **proportionally, with a floor of one**, which is what
    /// makes it mean "recently" rather than "ever". A flat drain would let a
    /// single enormous hit sit in the pool for ten seconds; a pure percentage
    /// never quite reaches zero in integers. Taking a percentage and at least
    /// one does both jobs and terminates.
    fn bleed_off(&mut self) {
        if self.strain > 0 {
            let drop = ((self.strain * self.sp().strain_decay()) / 100).max(1);
            self.strain = (self.strain - drop).max(0);
        }
        self.slowed = self.slowed.saturating_sub(1);
        // The Reaver's marks fade on the same clock a fighter's do.
        if self.marks > 0 {
            self.mark_clock = self.mark_clock.saturating_sub(1);
            if self.mark_clock == 0 {
                self.marks -= 1;
                if self.marks > 0 {
                    self.mark_clock = t::mark_fade();
                }
            }
        }
        if self.slowed == 0 {
            self.slow_mul = Fx::ONE;
        }
        self.rooted = self.rooted.saturating_sub(1);
        for slot in self.brain.cooldown.iter_mut() {
            *slot = slot.saturating_sub(1);
        }
    }
}
