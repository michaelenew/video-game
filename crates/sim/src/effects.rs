//! Things a move leaves behind.
//!
//! Until now every attack was an instant: a hitbox that existed for a few
//! frames and was gone. Two classes are built on the opposite idea — a fire
//! pillar that grows where you put it, a drain field that punishes standing
//! still. Those need to outlive the move that made them.
//!
//! **The Elementalist's structures are not in here**, and that is deliberate.
//! They are a cap-of-three resource owned by her mechanic, with no clock. Being
//! in this array gave them a lifetime and a second list to disagree with, and
//! the result was that her special silently stopped working ten seconds after
//! she pressed the button that enables it.
//!
//! **A fixed array, not a `Vec`.** Effects are simulation state, so they are
//! snapshotted and restored on every rollback; a heap allocation per frame of
//! re-simulation would be the most expensive thing in the tick. A hard cap also
//! makes "what happens when you spam it" a decision rather than an emergent
//! property: the oldest goes.
//!
//! ## Movement is a function of age, never a velocity
//!
//! Two of the five travel — a blade thrown out and caught again, four arms that
//! open into a cone and close to a point. A third, the fire tornado, travels in
//! a straight line rather than a curve, and still carries no velocity: where it
//! is is `pos` plus `dir` times `tornado_speed` times `age`, which is a formula
//! rather than a memory. None of the three carries a velocity. Where they are
//! is worked out from `age` every frame, so the whole flight is a pure function
//! of the frame the effect was cast on plus the frame it is now, and a rollback
//! that re-simulates the middle of a flight reproduces it exactly rather than
//! re-integrating it and landing somewhere near.

use crate::DT;
use crate::class::Class;
use crate::fixed::Fx;
use crate::math::V3;
use crate::moves::Move;
use crate::state::MAX_PLAYERS;
use crate::tuning as t;

/// How many effects can stand in the world at once.
///
/// Twelve, since the Blood mage's essence pools: she may keep `pool_cap` of
/// them on the floor, a mirror match keeps two of her, and each still wants a
/// blade in flight, a Grasp and a spike beside them. Eight was enough before
/// anything lived on the floor for as long as a pool does. Every slot costs
/// snapshot bytes and a hash line, which is why it is not simply large.
pub const MAX_EFFECTS: usize = 12;

/// Every effect standing in the world. Named so that the one bundle
/// `crate::aim` traces against can be written down without repeating the
/// array's shape at every call site.
pub type Effects = [Option<Effect>; MAX_EFFECTS];

/// Everything an effect can hit: both fighters, and the creature.
///
/// The creature gets a slot of its own rather than being squeezed in beside the
/// fighters because it is not one — it has parts, a hide and a poise bar. What
/// this index is for is only the bookkeeping of *what has already been hit*,
/// which is the same question for all three.
pub const VICTIMS: usize = MAX_PLAYERS + 1;
/// Where the creature sits in that numbering.
pub const QUARRY_VICTIM: usize = MAX_PLAYERS;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum EffectKind {
    /// Elementalist. Narrow and short at first, then grows: a wide, punishing
    /// base and a taller column above it.
    FirePillar,
    /// Elementalist. What a fire pillar becomes when Cataclysm passes through
    /// it instead of just charging the shot: the same two volumes -- see
    /// [`Effect::pillar_volumes`] -- cut loose from the ground they grew out
    /// of and sent racing along the line Cataclysm was aimed, pulling in
    /// anyone they pass over.
    ///
    /// **Not a new hitbox.** A pillar already standing is destroyed and a
    /// tornado spawned in its place once, back when Cataclysm was first
    /// built; this reuses the exact same `Effect` slot and the exact same
    /// two-volume shape instead, because the pillar's own hitbox is what a
    /// tornado wants to be, moving. See [`Effect::tornado_pos`] for the
    /// motion.
    FireTornado,
    /// Blood mage. A spike standing in a field that drains and slows anyone
    /// inside it, and feeds a share of what it drains back to the caster.
    BlackSpike,
    /// Blood mage. The auto: a blade thrown a fixed distance and caught again,
    /// cutting on both passes and bringing the blood home with it.
    Bloodletter,
    /// Blood mage. Four arms thrown out in a cone that arc back inward to meet
    /// at the far end. Caught by all four and you are rooted.
    Grasp,
    /// Blood mage. A bolt straight out along the crosshair, spent on the first
    /// body it reaches: a cut, and a **bleed** -- see `state::Player::bleeding`
    /// -- whose every tick spills a pool under the victim wherever they have
    /// got to. The easier thing to land in a kit where everything can miss,
    /// and what marks somebody as the target: a trail of pools behind them is
    /// a spike's chain pointed at them.
    Haemorrhage,
    /// Shadow Reaver. Six blades that erupt from the shadow along curving
    /// paths, hang at full extension, and then chase the shadow home -- cutting
    /// on the way out and again on the way back.
    ///
    /// The one effect whose centre **moves**, and it is the point of the
    /// ability rather than a detail: recall the shadow with the blades out and
    /// the twelve of them drag across the arena after it. See [`Effect::lotus_at`].
    GuillotineLotus,
    /// Dual mage, light Lance. What detonates where the line ran out.
    ///
    /// The line that carried it is the move's own hitbox and is a poke; this is
    /// the ability. That is what makes the light form a thing you aim **past**
    /// somebody: the damage is at the far end of the throw, so landing it means
    /// picking a point behind them rather than on them.
    LanceBurst,
    /// Dual mage, dark Lance. A line thrown along the crosshair that catches
    /// the first body it crosses and then **holds on**.
    ///
    /// Two effects in one lifetime, and the second is the ability. For the
    /// move's own active frames it is a line looking for something; once it has
    /// something it is a leash, draining what it caught every tick and parting
    /// the moment the two of them get further apart than
    /// `tuning::tether_leash`. A tether that held at any range would be a
    /// ranged tool on a melee kit; breaking on distance is what keeps her
    /// standing next to the thing she is draining.
    Tether,
    /// Dual mage, Judgement. The wide, low-damage ground the strike leaves
    /// behind it.
    ///
    /// Two halves pointing opposite ways, which is why it is one effect rather
    /// than two: it burns anybody standing in it, and it makes *her* fast while
    /// she is in it. So a Judgement thrown at somebody's feet is also a
    /// Judgement thrown at her own next few seconds, and how long either half
    /// lasts is how far out on the bar she was when she threw it.
    JudgementField,
    /// The other fighter's blood, pooled on the floor where the Blood mage cut
    /// them.
    ///
    /// **The class's object**, and the one effect in the game nothing casts:
    /// every hit she lands spills one under the point of contact, with a
    /// volume equal to the damage dealt (`banked`), and the only heal she has
    /// is putting a move through it -- see `state::World::drink`. It drains at
    /// a fixed volume a second so a big pool outlives a small one, merges
    /// with a pool it lands on, and is capped per owner at `pool_cap`. A slab
    /// on the floor, drawn at exactly the disc it is tested at; nothing
    /// collides with it, and standing in one is the point.
    /// See `docs/design/blood-mage.md` §"Essence pools".
    Pool,
}

/// How many arms a Grasp has, and which corner each one leaves by.
///
/// Top left, bottom left, top right, bottom right — the sign pair is
/// `(sideways, vertical)` against the line the ability was aimed along.
pub const GRASP_ARMS: usize = 4;
pub const GRASP_CORNERS: [(i32, i32); GRASP_ARMS] = [(-1, 1), (-1, -1), (1, 1), (1, -1)];

/// How many beads a tether is drawn as.
///
/// It is a **line**, and the renderer draws effects as pieces that have a place
/// and a size but no direction (`game::Piece`), so the line is a row of beads
/// rather than one long cylinder. That is a presentation decision that turned
/// out to be the right read as well: a chain visibly stretches as the two of
/// them part, and visibly is not there any more the frame the leash breaks.
///
/// Nothing in the simulation depends on it -- the hit test is a segment, once,
/// at the catch. See `Effect::tether_bead`.
pub const TETHER_BEADS: usize = 10;

/// How many blades a Guillotine lotus opens with.
///
/// Twelve, and a count rather than a knob for the same reason the Grasp has
/// four arms: it is what the ability *is*. A slider would be a second, worse
/// way of writing the move list.
///
/// It was six, of twice the size and twice the damage each. Six read as beach
/// balls rather than blades -- at a radius wider than a fighter's own, the
/// thing sweeping past was not shaped like anything you could call a blade.
/// Twelve smaller ones at half the damage is the same flower and the same total
/// if you eat all of it, made of pieces that look like what they are.
pub const LOTUS_BLADES: usize = 12;

/// The hit mask has room for every part of every effect against every victim.
///
/// Checked here because the failure is silent and looks like a gameplay bug: a
/// part whose bit falls off the end of the mask is a blade that can cut the
/// same person on every frame of its pass, or -- worse -- one that shares a bit
/// with another blade and goes quiet after that one lands. Going from six
/// blades to twelve is what made this worth stating: at six the mask needed 18
/// bits and a `u32` was fine, at twelve it needs 36 and was not.
const _: () = assert!(
    LOTUS_BLADES * VICTIMS <= u64::BITS as usize,
    "the hit mask cannot address every part against every victim"
);

/// Where in the hit mask the **pool** bits start: one per effect slot, so a
/// blade can remember which pools it has already drunk from on its way home.
/// The top of the word, above every (part, victim) bit a lotus can need.
const POOL_BITS: usize = u64::BITS as usize - MAX_EFFECTS;
const _: () = assert!(
    LOTUS_BLADES * VICTIMS <= POOL_BITS,
    "the pool bits overlap the hit bits"
);

impl EffectKind {
    pub const fn name(self) -> &'static str {
        match self {
            EffectKind::FirePillar => "fire pillar",
            EffectKind::FireTornado => "fire tornado",
            EffectKind::BlackSpike => "black spike",
            EffectKind::Bloodletter => "bloodletter",
            EffectKind::Grasp => "grasp",
            EffectKind::Haemorrhage => "haemorrhage",
            EffectKind::GuillotineLotus => "guillotine lotus",
            EffectKind::LanceBurst => "lance burst",
            EffectKind::Tether => "tether",
            EffectKind::JudgementField => "judgement field",
            EffectKind::Pool => "essence pool",
        }
    }

    /// Does this come out of the ground?
    ///
    /// A property of the thing rather than of the move that made it: a pillar
    /// of flame comes out of the floor whatever you were doing when you cast
    /// it. It decides where an aimed cast lands -- see `crate::aim::target`.
    ///
    /// A tornado is never itself the answer to that question -- nothing casts
    /// one this way, it only ever comes from a pillar that already stood --
    /// but the match has to say something, so it says what the pillar it came
    /// from would.
    pub const fn grounded(self) -> bool {
        match self {
            EffectKind::FirePillar | EffectKind::FireTornado | EffectKind::BlackSpike => true,
            // Both of these are thrown *through* the air along the line the
            // player is looking, so the crosshair means a direction rather than
            // a place on the floor.
            EffectKind::Bloodletter | EffectKind::Grasp | EffectKind::Haemorrhage => false,
            // Neither. It erupts at the shadow, wherever the shadow happens to
            // be standing -- which is the fourth line of effect, and the only
            // move in the game that uses it. See `aim::mechanic_path`.
            EffectKind::GuillotineLotus => false,
            // The burst goes off where the line stopped, which is a point in
            // the air rather than a patch of floor, and the tether is a line
            // through the air by construction. Judgement's field is the ground
            // the strike left, so it is the one of the three that is.
            EffectKind::LanceBurst | EffectKind::Tether => false,
            EffectKind::JudgementField => true,
            EffectKind::Pool => true,
        }
    }

    /// Does it stay where it was put?
    ///
    /// The two that do are places on the map and are drawn and tested where
    /// they were cast. The ones that do not work out where they are from their
    /// age -- see the module header.
    ///
    /// **The tornado is not one of these**, even though it moves: this
    /// question is about a one-shot pass that gets marked struck once landed
    /// (see `state::World::gore_the_creature`), and a tornado hits over and
    /// over on a tick the way the pillar it came from did, not once like a
    /// thrown blade.
    pub const fn travels(self) -> bool {
        matches!(
            self,
            EffectKind::Bloodletter
                | EffectKind::Grasp
                | EffectKind::Tether
                | EffectKind::Haemorrhage
        )
    }

    /// Does a grab in the move table mean anything for this effect?
    ///
    /// Only the Grasp, and only as the payoff for catching somebody with every
    /// one of its arms -- see the Grasp branch of `state::World::apply_effect`
    /// for why it cannot be per-arm. Everything else delivered by an effect
    /// carries damage, stun, blockstun, knockback and launch, and would drop a
    /// grab on the floor; the frame table says so rather than showing a knob
    /// nothing reads.
    pub const fn seizes(self) -> bool {
        matches!(self, EffectKind::Grasp)
    }

    /// Does its centre move after it is cast?
    ///
    /// One does. The lotus is anchored to the Reaver's shadow rather than to a
    /// patch of ground, so recalling the shadow takes the blades with it. Kept
    /// apart from [`travels`](Self::travels), which is about how the *cast* was
    /// aimed: what a lotus is thrown at is not a direction, and what it does
    /// after is not standing still.
    pub const fn follows_the_mechanic(self) -> bool {
        matches!(self, EffectKind::GuillotineLotus)
    }

    /// Does it come back to whoever threw it, rather than to where it was
    /// thrown from?
    ///
    /// One does. The Blood mage's blade is caught, and a catch is a thing that
    /// happens between two objects: a blade returning to a patch of air the
    /// caster has since walked out of has not been caught by anybody. Its
    /// `home` is refreshed from the owner every frame -- see
    /// `state::World::step_effects` -- and the return leg is drawn to it.
    pub const fn comes_home(self) -> bool {
        matches!(self, EffectKind::Bloodletter)
    }

    /// Which move index spawns this, for the move tables.
    pub const fn from_code(code: u8) -> Option<EffectKind> {
        match code {
            1 => Some(EffectKind::FirePillar),
            2 => Some(EffectKind::BlackSpike),
            3 => Some(EffectKind::Bloodletter),
            4 => Some(EffectKind::Grasp),
            5 => Some(EffectKind::GuillotineLotus),
            6 => Some(EffectKind::LanceBurst),
            7 => Some(EffectKind::Tether),
            8 => Some(EffectKind::JudgementField),
            // Listed so the numbering is complete; no move's row says it. A
            // pool is what a hit leaves, not what a cast places.
            9 => Some(EffectKind::Pool),
            10 => Some(EffectKind::Haemorrhage),
            _ => None,
        }
    }

    /// How long one of these lives, in frames.
    ///
    /// **A tornado never actually asks this.** It is never cast fresh through
    /// [`Effect::cast`] -- it only ever comes from mutating a standing
    /// `FirePillar` in place, `age` and `life` both left exactly as they
    /// were, so what is left of the pillar's own life is what it gets to
    /// travel on. The arm exists because the match still has to answer for
    /// every kind; it gives the pillar's own number, since that is the
    /// closest thing to a true answer.
    pub fn life(self) -> u16 {
        match self {
            EffectKind::FirePillar | EffectKind::FireTornado => t::pillar_life(),
            EffectKind::BlackSpike => t::spike_erupt(),
            EffectKind::Bloodletter => t::bloodletter_flight(),
            EffectKind::Grasp => t::grasp_flight(),
            EffectKind::Haemorrhage => t::haemorrhage_flight(),
            // Out, held open, and home again. Three knobs rather than one
            // because they are three different decisions: how fast it opens is
            // spectacle, how long it hangs is how much time the victim has to
            // leave, and how long it takes to come back is how far the blades
            // can drag a recalled shadow.
            EffectKind::GuillotineLotus => t::lotus_erupt()
                .saturating_add(t::lotus_hold())
                .saturating_add(t::lotus_return()),
            EffectKind::LanceBurst => t::lance_burst_life(),
            EffectKind::Tether => t::tether_life(),
            EffectKind::JudgementField => t::judgement_field_life(),
            // A pool has no clock. It is gone when it has drained, which is
            // volume, not frames -- see `state::World::step_effects`.
            EffectKind::Pool => u16::MAX,
        }
    }

    /// What it deals each time it connects.
    ///
    /// The two that stay put have a number of their own, because the move that
    /// placed them **also** hit on its own at the moment of casting and the two
    /// are not the same event: a fire pillar's eruption is not its burn. The
    /// two that travel take the move's, because for those the effect *is* the
    /// hit — the caster's body never touches anybody.
    ///
    /// The tornado takes the pillar's own number rather than a number of its
    /// own -- it is the same fire, still burning on the same tick, now also
    /// dragging you toward it. What is new about it is the pull, not the burn.
    pub fn damage(self, m: &Move) -> i32 {
        match self {
            EffectKind::FirePillar | EffectKind::FireTornado => t::pillar_damage(),
            EffectKind::BlackSpike => m.damage,
            EffectKind::Bloodletter
            | EffectKind::Grasp
            | EffectKind::GuillotineLotus
            | EffectKind::Haemorrhage => m.damage,
            // All three of the Dual mage's carry a number of their own, for the
            // same reason a pillar does: the move that threw them hit on its
            // own and the two are not the same event. The light Lance's line is
            // a poke and its burst is the ability; Judgement's strike is the
            // damage and its field is the ground it left; and the dark Lance
            // has no strike at all, so the drain is all there is.
            EffectKind::LanceBurst => t::lance_burst_damage(),
            EffectKind::Tether => t::tether_drain(),
            EffectKind::JudgementField => t::judgement_field_damage(),
            EffectKind::Pool => 0,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Effect {
    pub kind: EffectKind,
    /// Who made it. An effect never hurts its owner.
    pub owner: u8,
    /// Which move made it.
    ///
    /// Kept rather than copying out the half-dozen numbers an effect needs,
    /// because a detached hit wants everything an attached one has: how far it
    /// goes, what it deals, how long it stuns, whether guard stops it. One
    /// lookup answers all of that from the same table the move itself reads, so
    /// there is no second copy of an ability's numbers to disagree with the
    /// first.
    pub class: Class,
    pub slot: u8,
    /// Where it was cast. For the two that stay put this is where they are; for
    /// the two that travel it is where they left from and where they come back
    /// to.
    pub pos: V3,
    /// The line it was thrown along, as a unit vector. Zero for the two that do
    /// not go anywhere.
    pub dir: V3,
    /// Frames since it appeared. Growth is a function of this, so it is a pure
    /// function of the snapshot and rollback reproduces it exactly.
    pub age: u16,
    pub life: u16,
    /// Who has already been hit, and by which part.
    ///
    /// One bit per (part, victim). A blade has two parts — the pass out and the
    /// pass back — and clears the mask between them, so it can catch the same
    /// person twice. A Grasp has four, one per arm, and never clears: an arm
    /// hits you once, and how many *different* arms have is exactly the
    /// question the root is asking. A lotus has twelve, and clears at the turn
    /// the way the blade does.
    ///
    /// A `u64`, and the lotus is what costs it: twelve parts against three
    /// victims is thirty-six bits. It was a `u32` at six blades, which fitted
    /// exactly and would have silently truncated at twelve -- see the
    /// assertion beside [`LOTUS_BLADES`], which is there so the next count
    /// change fails to compile rather than failing to cut.
    pub struck: u64,
    /// How far this one travels, which is normally the move's own `reach`.
    ///
    /// On the row for every effect rather than read back off the move, because
    /// a channelled move does not have one answer: the Grasp's arms converge
    /// wherever the caster wound the aim marker to, and the marker is gone by
    /// the time the arms exist. See `state::step_channel`.
    pub reach: Fx,
    /// Where a returning effect is heading, live.
    ///
    /// The caster's ability origin for anything that [`comes_home`], refreshed
    /// every frame, and the cast position for everything else -- so an effect
    /// nobody updates flies exactly the arc it always did.
    ///
    /// Separate from `pos`, which for the blade stays the point it was thrown
    /// from: the *outward* leg is the line the crosshair picked and moving it
    /// after the throw would be aiming an ability that is already out. Only the
    /// way home follows anybody.
    ///
    /// [`comes_home`]: EffectKind::comes_home
    pub home: V3,
    /// Damage this has dealt and not yet paid back.
    ///
    /// Only the blade uses it. The archive is specific that the health arrives
    /// **when it returns**, which is the whole risk of the ability: the cut
    /// lands immediately and the payment has to survive the flight home.
    ///
    /// A fire tornado banks something else in the same field: `age` the
    /// instant it was cut loose, so [`tornado_pos`](Self::tornado_pos) can
    /// measure its flight from that moment on rather than from when the
    /// pillar it came from was first planted. See `state::World::fire_the_cataclysm`.
    pub banked: i32,
    /// What the caster's bar was worth when this was thrown, as a multiplier.
    ///
    /// [`Fx::ONE`] for five of the six classes, and the Dual mage's depth curve
    /// for the sixth -- see `state::depth`. It scales what this deals and how
    /// big it is, so a Judgement thrown from the centre leaves a puddle and one
    /// thrown from the edge leaves the biggest thing in the game.
    ///
    /// **Fixed at the cast, not read live**, and that is the overlay's rule
    /// rather than a design preference: `field_radius` is what the hit test
    /// uses *and* what the renderer draws, so it has to be a number the
    /// renderer can get at without knowing where the caster is standing this
    /// frame. A size that changed under a thing already in the world would also
    /// be a hazard that moved after you decided to walk round it.
    pub power: Fx,
}

impl Effect {
    /// Put one into the world.
    pub fn cast(
        kind: EffectKind,
        owner: u8,
        class: Class,
        slot: u8,
        pos: V3,
        dir: V3,
        reach: Fx,
    ) -> Effect {
        Effect {
            kind,
            owner,
            class,
            slot,
            pos,
            dir,
            age: 0,
            life: kind.life().max(1),
            struck: 0,
            reach,
            home: pos,
            banked: 0,
            power: Fx::ONE,
        }
    }

    /// The move that made this.
    pub fn source(&self) -> Move {
        crate::moves::get(self.class, self.slot)
    }

    /// What this effect does to whatever it touches, over and above damage.
    ///
    /// One place rather than three, because the creature is offered the same
    /// control a fighter gets and the two should never be able to disagree
    /// about what a Black spike does. The fighter path still applies its own
    /// separately: a fighter takes the whole of it and a monster does not --
    /// see `monster::Monster::take_control`.
    pub fn control(&self) -> crate::monster::Control {
        use crate::monster::Control;
        use crate::tuning as t;
        match self.kind {
            EffectKind::BlackSpike => Control::slowing(t::slow_frames(), t::spike_slow()),
            EffectKind::GuillotineLotus => Control::slowing(t::slow_frames(), t::lotus_slow()),
            EffectKind::Grasp => Control::grabbing(self.source().grabs),
            _ => Control::default(),
        }
    }

    /// What it deals each time it connects, at the power it was thrown with.
    pub fn damage(&self) -> i32 {
        Fx::from_int(self.kind.damage(&self.source()))
            .mul(self.power)
            .to_int()
    }

    /// Percent of what it deals that goes back to the caster.
    pub fn leech(&self) -> u8 {
        self.source().leech
    }

    /// Health owed for `dealt` damage.
    pub fn leeched(&self, dealt: i32) -> i32 {
        self.source().leeched(dealt)
    }

    /// How far through its life, as a fraction.
    ///
    /// Fixed point, so growth curves are deterministic. Saturates at one rather
    /// than running past, since an effect is removed the frame it expires.
    pub fn progress(&self) -> Fx {
        if self.life == 0 {
            return Fx::ONE;
        }
        let p = Fx::from_int(self.age as i32).div(Fx::from_int(self.life as i32));
        if p.raw() > Fx::ONE.raw() { Fx::ONE } else { p }
    }

    /// The two volumes a fire pillar occupies: a wide base and a taller, only
    /// slightly wider column above it.
    ///
    /// Two rather than one because they are different threats. The base is what
    /// catches someone walking into it; the column is what stops them jumping
    /// over. A single growing cylinder would make those one decision, and the
    /// pillar would be either useless against a jump or unavoidable on the
    /// ground.
    ///
    /// **A tornado grows on the same curve its pillar was already growing
    /// on**, against `age` left exactly as it was the instant Cataclysm cut
    /// it loose -- see `state::World::fire_the_cataclysm`, which never
    /// resets it. It is not a new thing that starts moving already finished:
    /// one caught early keeps erupting into itself while it travels, the
    /// same growth a planted pillar would have shown standing still, and one
    /// caught late is already as grown as it was ever going to get and stays
    /// that way. How long it then gets to keep existing is a different
    /// question, asked apart from this one -- see `tuning::tornado_travel_life`.
    pub fn pillar_volumes(&self) -> (Pillar, Pillar) {
        let grown = self.progress();
        let base = Pillar {
            radius: lerp(
                t::pillar_base_radius_start(),
                t::pillar_base_radius(),
                grown,
            ),
            bottom: Fx::ZERO,
            top: t::pillar_base_height(),
        };
        // The column takes the top four fifths, and widens far less than the
        // base does -- it grows *up* rather than *out*.
        let column = Pillar {
            radius: lerp(t::pillar_radius_start(), t::pillar_column_radius(), grown),
            bottom: t::pillar_base_height(),
            top: lerp(t::pillar_height_start(), t::pillar_height(), grown),
        };
        (base, column)
    }

    /// Where a tornado's centre is this frame.
    ///
    /// A pure function of `age`, not a velocity: `pos` is where the pillar it
    /// was cut loose from stood -- which does not move, so it is equally
    /// where the tornado started -- `dir` is the line Cataclysm was aimed
    /// along, and how far past `pos` it has arrived is *however many frames
    /// have passed since it was cut loose* of `tornado_speed`. That is `age`
    /// minus `banked`, not `age` alone: `age` keeps counting from when the
    /// pillar was first planted, because [`pillar_volumes`](Self::pillar_volumes)
    /// needs the growth to be continuous across the moment it starts moving,
    /// but the flight cannot be measured against the same clock a stationary
    /// pillar was already running -- `state::World::fire_the_cataclysm` banks
    /// the age it had at that exact moment, and this subtracts it back off. A
    /// rollback re-simulating the middle of its flight computes the same
    /// point every time it asks, rather than integrating toward it one frame
    /// at a time and landing somewhere near. Meaningless -- and never called
    /// -- on anything but a [`EffectKind::FireTornado`].
    pub fn tornado_pos(&self) -> V3 {
        let flying = self.age.saturating_sub(self.banked as u16);
        let travelled = t::tornado_speed().mul(Fx::from_int(flying as i32)).mul(DT);
        self.pos.add(self.dir.scale(travelled))
    }

    /// The volume a black spike occupies: a disc of ground with a spike
    /// standing in the middle of it.
    ///
    /// A slab rather than a flat circle, so that **a drain field can be jumped
    /// over**. It used to be tested on flat distance alone, which made it the
    /// one hazard in the game with infinite height -- you could be draining
    /// three metres above it at the top of a jump. Height decides a pillar and
    /// it decides this, for the same reason: what you can see is a thing of a
    /// certain size, and the hit test should agree with your eyes.
    pub fn spike_volume(&self) -> Pillar {
        Pillar {
            // The disc it came out of: the move's own radius on bare floor,
            // the eruption's when it came up out of a pool. See
            // `state::World::advance`.
            radius: self.reach,
            bottom: Fx::ZERO,
            top: if self.erupted() {
                t::erupt_height()
            } else {
                t::spike_height()
            },
        }
    }

    /// Did this spike come up out of a pool, at the pool's size? A bare-floor
    /// spike did its hitting through the move's own disc and is only a thing
    /// to look at; an erupted one delivers the hit itself, once.
    pub const fn erupted(&self) -> bool {
        matches!(self.kind, EffectKind::BlackSpike) && self.banked > 0
    }

    /// Radius of a field effect. Drain fields do not grow; they are a place.
    pub fn field_radius(&self) -> Fx {
        self.own_radius().mul(self.power)
    }

    /// The radius before the caster's own power is applied. Split out only
    /// because the two are one multiply and the match is long.
    fn own_radius(&self) -> Fx {
        match self.kind {
            EffectKind::BlackSpike => self.reach,
            EffectKind::FirePillar | EffectKind::FireTornado => self.pillar_volumes().0.radius,
            EffectKind::Bloodletter => t::bloodletter_radius(),
            EffectKind::Grasp => t::grasp_arm_radius(),
            EffectKind::Haemorrhage => t::haemorrhage_radius(),
            EffectKind::GuillotineLotus => t::lotus_blade_radius(),
            EffectKind::LanceBurst => t::lance_burst_radius(),
            // How thick the thrown line is, read off the move's own row -- the
            // dark Lance puts no volume out of its body (`moves::shape` answers
            // `None`), so its `radius` is free to be the one thing about the
            // line that is a size, and it is where a tuner would look for it.
            EffectKind::Tether => self.source().radius,
            EffectKind::JudgementField => t::judgement_field_radius(),
            EffectKind::Pool => self.pool_radius(),
        }
    }

    /// Whether this effect damages on a given frame.
    ///
    /// A field that hit every frame would do sixty times its listed damage a
    /// second, so damage-over-time ticks on a cadence and the number in the
    /// move table is per tick.
    pub fn ticks_now(&self) -> bool {
        let every = t::effect_tick_frames().max(1);
        self.age > 0 && self.age % every == 0
    }

    // -- The two that travel ------------------------------------------------

    /// True once the blade has turned and is on its way home.
    pub fn returning(&self) -> bool {
        self.age as u32 * 2 > self.life as u32
    }

    /// Which pass the blade is on: nought out, one back.
    pub fn pass(&self) -> usize {
        self.returning() as usize
    }

    /// The far end of the blade's throw: where it turns round.
    fn blade_apex(&self) -> V3 {
        self.pos.add(self.dir.scale(self.reach))
    }

    /// Where the blade is this frame.
    ///
    /// Two straight lines with the turn between them, and they are not the same
    /// kind of line. **Out** is the throw: from where it left to the far end of
    /// the reach the crosshair picked, and nothing moves it. **Back** is the
    /// catch: from the far end to [`home`](Effect::home), which is wherever the
    /// caster is *now*, so a mage who walks while the blade is in the air has
    /// it curve after them and arrive in their hand anyway.
    ///
    /// Straight lines rather than a curve on purpose. A thrown blade that eased
    /// in and out would hang at the far end, and hanging is what a *placed*
    /// effect does; this one is meant to read as a single continuous throw
    /// whose only event is the turn.
    ///
    /// **The return is a fraction of the way home rather than a speed**, which
    /// is what makes it arrive on the frame it is supposed to however far the
    /// caster has run. A chase at a fixed speed could be outrun, and then the
    /// ability's failure case would be *moving* -- which is the one thing an
    /// aggressive sustain class has to be able to do. The risk stays where the
    /// design put it: you have to survive the flight, not stand still for it.
    pub fn blade_at(&self) -> V3 {
        let p = self.progress();
        let half = Fx::ratio(1, 2);
        let apex = self.blade_apex();
        if p.raw() <= half.raw() {
            crate::math::lerp3(self.pos, apex, p.mul(Fx::from_int(2)))
        } else {
            crate::math::lerp3(apex, self.home, p.sub(half).mul(Fx::from_int(2)))
        }
    }

    /// Where the Haemorrhage's bolt is this frame: straight out along the line
    /// it was thrown, its whole reach over its whole life.
    pub fn bolt_at(&self) -> V3 {
        self.pos
            .add(self.dir.scale(self.reach.mul(self.progress())))
    }

    /// Where one arm of a Grasp is this frame.
    ///
    /// Along the aim by however far through the throw it is, and off it by a
    /// bulge that is zero at both ends and widest halfway. `4p(1 - p)` is the
    /// cheapest curve with exactly that shape, and it needs no trigonometry, so
    /// the arms are the same on every machine.
    pub fn arm_at(&self, arm: usize) -> V3 {
        let p = self.progress();
        let bulge = Fx::from_int(4).mul(p).mul(Fx::ONE.sub(p));
        let (side, up) = GRASP_CORNERS[arm.min(GRASP_ARMS - 1)];
        let (right, lift) = crate::math::frame_about(self.dir);
        let spread = t::grasp_spread().mul(bulge);
        self.pos
            .add(self.dir.scale(self.reach.mul(p)))
            .add(right.scale(spread.mul(Fx::from_int(side))))
            .add(lift.scale(spread.mul(Fx::from_int(up))))
    }

    // -- The lotus ----------------------------------------------------------

    /// Which of the three parts of a lotus this frame is in, and how far
    /// through that part it is.
    ///
    /// Three parts rather than one curve because they are three different
    /// motions and a player has to be able to tell them apart: the blades
    /// **snap** out, **hang** open, and then **slide** home decelerating. A
    /// single ease over the whole life would blur all three into one breath.
    pub fn lotus_phase(&self) -> LotusPhase {
        self.lotus_phase_at(self.age)
    }

    /// The same question asked of any frame, so the sweep below can ask about
    /// the one just gone without building a second, near-identical `Effect`.
    fn lotus_phase_at(&self, age: u16) -> LotusPhase {
        let erupt = t::lotus_erupt().max(1);
        let hold = t::lotus_hold();
        let back = t::lotus_return().max(1);
        if age < erupt {
            LotusPhase::Erupting(Fx::ratio(age as i32, erupt as i32))
        } else if age < erupt + hold {
            LotusPhase::Held
        } else {
            let through = age - erupt - hold;
            LotusPhase::Returning(Fx::ratio(through.min(back) as i32, back as i32))
        }
    }

    /// How far out a blade is and how far round it has turned, at one age.
    ///
    /// **The two come out together and that is the point.** A blade's path is
    /// one motion -- reach and bearing advancing on the same curve -- so a
    /// spiral is what you get rather than something you add. Split into a
    /// radius here and an angle somewhere else, the two drift apart the first
    /// time either ease is touched.
    ///
    /// Reach is out on a curve that is fastest at the start, because the
    /// eruption is meant to be over before the victim can answer it; home on
    /// the mirror of that, which is what "slowing" means -- the blades come off
    /// full speed and settle into the shadow rather than snapping back.
    ///
    /// The turn is **not** symmetric, and that is the whole difference between
    /// a flower closing and a flower rewinding. Going out a blade turns
    /// `lotus_curl` in the direction it opened. Coming home it turns
    /// `lotus_uncurl` the *other* way, which is more, so it carries on past the
    /// bearing it started from and closes on a spiral of its own instead of
    /// retracing the arm it came out on. See `tuning::lotus_uncurl` for why
    /// that is a hit test and not only a look.
    fn lotus_reach_and_turn(&self, age: u16) -> (Fx, Fx) {
        match self.lotus_phase_at(age) {
            // Full speed on the first frame, arriving at rest.
            LotusPhase::Erupting(p) => {
                let out = crate::math::ease_out(p);
                (out, t::lotus_curl().mul(out))
            }
            LotusPhase::Held => (Fx::ONE, t::lotus_curl()),
            // The reach runs the outward curve backwards; the turn does not.
            // Both are driven off the same `u` so they stay one motion.
            LotusPhase::Returning(p) => {
                let u = crate::math::ease_out(p);
                (
                    Fx::ONE.sub(u),
                    t::lotus_curl().sub(t::lotus_uncurl().mul(u)),
                )
            }
        }
    }

    /// The line one blade swept this frame: where its head was, and where it is.
    ///
    /// **A segment rather than a point, and that is a hit test rather than a
    /// flourish.** The eruption crosses four metres in seven frames and it is
    /// fastest on the first of them, so a head tested as a ball starts the
    /// frame at the shadow's feet and ends it a metre past anybody standing
    /// there -- the one victim the ability is named for is the one it would
    /// miss. A swept line cannot tunnel.
    ///
    /// Both ends are taken around the **current** centre. The shadow's own
    /// motion is not swept, and does not need to be: it moves less than a
    /// blade's own thickness in a frame, so a body it crosses is inside one
    /// end or the other on every frame of the crossing.
    pub fn lotus_span(&self, blade: usize, centre: V3) -> (V3, V3) {
        (
            self.lotus_head(blade, centre, self.age.saturating_sub(1)),
            self.lotus_at(blade, centre),
        )
    }

    /// Where one blade is this frame, around a centre the caller supplies.
    ///
    /// **The centre is passed in rather than read from `pos`** because it is
    /// the shadow's live position, and the shadow moves: recall it with the
    /// blades out and the twelve of them are dragged across the arena behind it,
    /// which is most of what the ability is for.
    ///
    /// Each blade leaves on its own twelfth of the circle and keeps turning as
    /// it goes, so the twelve open like petals rather than as spokes of a
    /// wheel -- and they come home turning the other way, past where they
    /// started. See [`Effect::lotus_reach_and_turn`].
    pub fn lotus_at(&self, blade: usize, centre: V3) -> V3 {
        self.lotus_head(blade, centre, self.age)
    }

    /// One blade's head at a given age. The shape of the flower, taking the age
    /// rather than reading `self.age`, so the sweep above can ask for two
    /// frames at once.
    ///
    /// **Flat.** Every blade sits `lotus_height` above the shadow's feet for the
    /// whole of its life: the flower opens, holds and closes in one horizontal
    /// plane. It used to arc up and back down over the eruption, which left the
    /// blade half buried in the floor at exactly the reach where it does the
    /// most work -- see `tuning::lotus_height`.
    fn lotus_head(&self, blade: usize, centre: V3, age: u16) -> V3 {
        let (out, turned) = self.lotus_reach_and_turn(age);
        let bearing =
            Fx::ratio(blade.min(LOTUS_BLADES - 1) as i32, LOTUS_BLADES as i32).add(turned);
        let along = V3::from_turns(bearing);
        let reach = t::lotus_radius().mul(out);
        V3::new(
            centre.x.add(along.x.mul(reach)),
            centre.y.add(t::lotus_height()),
            centre.z.add(along.z.mul(reach)),
        )
    }

    /// The middle of the flower: the centre raised into the plane the blades
    /// lie in.
    ///
    /// Exists for the overlay, which draws a spoke from here out to each head.
    /// Asked of the simulation rather than worked out beside the renderer,
    /// because a plane the overlay believes in and a plane the hit test sweeps
    /// are the same plane or the overlay is lying about the thing it is there
    /// to illustrate.
    pub fn lotus_hub(&self, centre: V3) -> V3 {
        V3::new(centre.x, centre.y.add(t::lotus_height()), centre.z)
    }

    /// True once the blades have turned for home.
    pub fn lotus_coming_back(&self) -> bool {
        matches!(self.lotus_phase(), LotusPhase::Returning(_))
    }

    /// Cut the return short and send the blades home now.
    ///
    /// Recalling the shadow reactivates the lotus, which is the combination the
    /// kit is built around: the blades stop hanging and start chasing. It winds
    /// the clock forward to the first frame of the return rather than shortening
    /// the life, so the drag home is always the full `lotus_return`.
    pub fn lotus_send_home(&mut self) {
        let turn = t::lotus_erupt().max(1).saturating_add(t::lotus_hold());
        if self.age < turn {
            self.age = turn;
            self.forget_hits();
        }
    }

    // -- Bookkeeping --------------------------------------------------------

    fn bit(part: usize, victim: usize) -> u64 {
        1u64 << (part * VICTIMS + victim)
    }

    /// Has this part already caught this victim?
    pub fn already_hit(&self, part: usize, victim: usize) -> bool {
        self.struck & Effect::bit(part, victim) != 0
    }

    /// Mark a victim hit by one part, and say whether it was already.
    pub fn take_hit(&mut self, part: usize, victim: usize) -> bool {
        if self.already_hit(part, victim) {
            return false;
        }
        self.struck |= Effect::bit(part, victim);
        true
    }

    /// How many different parts have caught this victim.
    pub fn parts_landed(&self, victim: usize, parts: usize) -> usize {
        (0..parts)
            .filter(|part| self.already_hit(*part, victim))
            .count()
    }

    /// Where one bead of a tether is drawn, of [`TETHER_BEADS`] along it.
    ///
    /// From `pos` to `home`, both of which are kept live by
    /// `state::World::step_effects`: `pos` is the caster's hand this frame and
    /// `home` is either the far end of the throw, while the line is still
    /// looking for something, or the thing it caught. So the one way to read
    /// "it took hold" is that the far end stopped being where you aimed and
    /// started being a person.
    pub fn tether_bead(&self, i: usize) -> V3 {
        let last = TETHER_BEADS.saturating_sub(1).max(1);
        crate::math::lerp3(self.pos, self.home, Fx::ratio(i as i32, last as i32))
    }

    /// Forget everyone hit so far, so the next pass starts clean.
    pub fn forget_hits(&mut self) {
        self.struck = 0;
    }

    // -- Essence pools -----------------------------------------------------

    /// A pool of `volume`, spilled on the floor at `at` by `owner`.
    ///
    /// `slot` is the move that spilled it, kept for the same reason every
    /// effect keeps one: a reader asking what made this is answered from the
    /// table rather than from a copy.
    pub fn pool(owner: u8, class: Class, slot: u8, at: V3, volume: i32) -> Effect {
        Effect {
            kind: EffectKind::Pool,
            owner,
            class,
            slot,
            pos: at,
            dir: V3::ZERO,
            age: 0,
            life: u16::MAX,
            struck: 0,
            reach: Fx::ZERO,
            home: at,
            banked: volume,
            power: Fx::ONE,
        }
    }

    pub const fn is_a_pool(&self) -> bool {
        matches!(self.kind, EffectKind::Pool)
    }

    /// How much blood is in it. What a drink takes from and what the drain
    /// takes off.
    pub const fn pool_volume(&self) -> i32 {
        self.banked
    }

    /// How much of a body this pool is, nought to one: a full figure at
    /// `tuning::pool_full` of essence, never less than `tuning::pool_least`.
    /// It shrinks as it drains, and the renderer draws it more solid the more
    /// is left -- which is the whole of how a pool is read.
    pub fn pool_share(&self) -> Fx {
        Fx::ratio(self.banked.clamp(0, t::pool_full()), t::pool_full()).max(t::pool_least())
    }

    /// How wide the figure is: a body's radius, by its share.
    pub fn pool_radius(&self) -> Fx {
        t::body_radius().mul(self.pool_share())
    }

    /// How tall it stands: a body's height, by its share.
    pub fn pool_height(&self) -> Fx {
        t::body_height().mul(self.pool_share())
    }

    /// The column a pool is tested as: a shadowy figure standing on the floor
    /// where the blood was spilled, a body's size at most and shrinking as it
    /// drains. Drawn at exactly this.
    pub fn pool_slab(&self) -> Pillar {
        Pillar {
            radius: self.pool_radius(),
            bottom: Fx::ZERO,
            top: self.pool_height(),
        }
    }

    /// Is a body standing at `at` -- a pair of feet, the centre of a hit --
    /// touching this figure? A body's radius of slack, so *in* a pool means
    /// against it, not inside a column narrower than a foot.
    pub fn covers(&self, at: V3) -> bool {
        let flat = V3::new(at.x.sub(self.pos.x), Fx::ZERO, at.z.sub(self.pos.z)).flat_len();
        flat.raw() <= self.pool_radius().add(t::body_radius()).raw()
            && at.y.raw() >= self.pos.y.sub(self.pool_height()).raw()
            && at.y.raw() <= self.pos.y.add(self.pool_height()).raw()
    }

    /// How much this pool loses this frame: `tuning::pool_drain` a second,
    /// spread over the frames by the same integer trick the grey fade uses, so
    /// a second of draining is exactly the rate whatever the rounding did in
    /// between.
    pub fn pool_drained_this_frame(&self) -> i32 {
        let rate = t::pool_drain().max(0);
        let hz = crate::TICK_HZ as i32;
        let k = (self.age as u32 % crate::TICK_HZ) as i32;
        rate * (k + 1) / hz - rate * k / hz
    }

    /// Has this effect already drunk from the pool in `slot`? A blade crossing
    /// pools on its way home drinks from each one once.
    pub fn drank_from(&self, slot: usize) -> bool {
        self.struck & (1u64 << (POOL_BITS + slot.min(MAX_EFFECTS - 1))) != 0
    }

    pub fn mark_drank(&mut self, slot: usize) {
        self.struck |= 1u64 << (POOL_BITS + slot.min(MAX_EFFECTS - 1));
    }

    /// Who a tether has hold of, if anything.
    ///
    /// The hit mask is doing double duty here and it is the right field for it:
    /// a tether has one part and catches one body, so "who has this part hit"
    /// and "who is on the end of the line" are the same question. No extra
    /// field on an effect that is copied on every rollback frame, and nothing
    /// to keep in step with the mask.
    ///
    /// Fighters only. The creature is caught by the line's first pass like
    /// anything else, but nothing holds a Ridgeback on a leash.
    pub fn caught(&self) -> Option<usize> {
        (0..crate::state::MAX_PLAYERS).find(|i| self.already_hit(0, *i))
    }
}

/// Which part of its life a Guillotine lotus is in.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LotusPhase {
    /// Blades racing outward, with the fraction of the way through the burst.
    Erupting(Fx),
    /// Open, and still.
    Held,
    /// Chasing the shadow home, with the fraction of the way through the drag.
    Returning(Fx),
}

/// One cylindrical slab of a pillar.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Pillar {
    pub radius: Fx,
    pub bottom: Fx,
    pub top: Fx,
}

impl Pillar {
    /// Whether a fighter standing at `pos` with the given body is inside.
    ///
    /// Flat distance against the radius, and a vertical overlap test against
    /// the slab — which is the one place in the game where height genuinely
    /// decides a hit, and the reason a pillar can be jumped over while a sword
    /// cannot be ducked.
    pub fn contains(&self, centre: V3, pos: V3, body_radius: Fx, body_height: Fx) -> bool {
        let flat = V3::new(pos.x.sub(centre.x), Fx::ZERO, pos.z.sub(centre.z)).flat_len();
        if flat.raw() > self.radius.add(body_radius).raw() {
            return false;
        }
        let feet = pos.y;
        let head = pos.y.add(body_height);
        head.raw() >= centre.y.add(self.bottom).raw() && feet.raw() <= centre.y.add(self.top).raw()
    }
}

fn lerp(from: Fx, to: Fx, at: Fx) -> Fx {
    from.add(to.sub(from).mul(at))
}
