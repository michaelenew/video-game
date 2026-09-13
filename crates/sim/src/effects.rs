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

pub const MAX_EFFECTS: usize = 8;

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
    /// Shadow Reaver. Six blades that erupt from the shadow along curving
    /// paths, hang at full extension, and then chase the shadow home -- cutting
    /// on the way out and again on the way back.
    ///
    /// The one effect whose centre **moves**, and it is the point of the
    /// ability rather than a detail: recall the shadow with the blades out and
    /// the six of them drag across the arena after it. See [`Effect::lotus_at`].
    GuillotineLotus,
}

/// How many arms a Grasp has, and which corner each one leaves by.
///
/// Top left, bottom left, top right, bottom right — the sign pair is
/// `(sideways, vertical)` against the line the ability was aimed along.
pub const GRASP_ARMS: usize = 4;
pub const GRASP_CORNERS: [(i32, i32); GRASP_ARMS] = [(-1, 1), (-1, -1), (1, 1), (1, -1)];

/// How many blades a Guillotine lotus opens with.
///
/// Six, and a count rather than a knob for the same reason the Grasp has four
/// arms: it is what the ability *is*. A slider from one to twelve would be a
/// second, worse way of writing the move list, and the bookkeeping below packs
/// one bit per blade per victim into a `u32`, which six of them and three
/// victims exactly fits.
pub const LOTUS_BLADES: usize = 6;

impl EffectKind {
    pub const fn name(self) -> &'static str {
        match self {
            EffectKind::FirePillar => "fire pillar",
            EffectKind::FireTornado => "fire tornado",
            EffectKind::BlackSpike => "black spike",
            EffectKind::Bloodletter => "bloodletter",
            EffectKind::Grasp => "grasp",
            EffectKind::GuillotineLotus => "guillotine lotus",
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
            EffectKind::Bloodletter | EffectKind::Grasp => false,
            // Neither. It erupts at the shadow, wherever the shadow happens to
            // be standing -- which is the fourth line of effect, and the only
            // move in the game that uses it. See `aim::mechanic_path`.
            EffectKind::GuillotineLotus => false,
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
        matches!(self, EffectKind::Bloodletter | EffectKind::Grasp)
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

    /// Which move index spawns this, for the move tables.
    pub const fn from_code(code: u8) -> Option<EffectKind> {
        match code {
            1 => Some(EffectKind::FirePillar),
            2 => Some(EffectKind::BlackSpike),
            3 => Some(EffectKind::Bloodletter),
            4 => Some(EffectKind::Grasp),
            5 => Some(EffectKind::GuillotineLotus),
            _ => None,
        }
    }

    /// How long one of these lives, in frames.
    pub fn life(self) -> u16 {
        match self {
            EffectKind::FirePillar => t::pillar_life(),
            EffectKind::FireTornado => t::tornado_life(),
            EffectKind::BlackSpike => t::spike_life(),
            EffectKind::Bloodletter => t::bloodletter_flight(),
            EffectKind::Grasp => t::grasp_flight(),
            // Out, held open, and home again. Three knobs rather than one
            // because they are three different decisions: how fast it opens is
            // spectacle, how long it hangs is how much time the victim has to
            // leave, and how long it takes to come back is how far the blades
            // can drag a recalled shadow.
            EffectKind::GuillotineLotus => t::lotus_erupt()
                .saturating_add(t::lotus_hold())
                .saturating_add(t::lotus_return()),
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
            EffectKind::BlackSpike => t::spike_drain(),
            EffectKind::Bloodletter | EffectKind::Grasp | EffectKind::GuillotineLotus => m.damage,
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
    /// question the root is asking. A lotus has six, and clears at the turn the
    /// way the blade does.
    ///
    /// A `u32` rather than a `u16`, which is what the lotus cost: six parts
    /// against three victims is eighteen bits and a `u16` holds five parts.
    pub struck: u32,
    /// How far this one travels, which is normally the move's own `reach`.
    ///
    /// On the row for every effect rather than read back off the move, because
    /// a channelled move does not have one answer: the Grasp's arms converge
    /// wherever the caster wound the aim marker to, and the marker is gone by
    /// the time the arms exist. See `state::step_channel`.
    pub reach: Fx,
    /// Damage this has dealt and not yet paid back.
    ///
    /// Only the blade uses it. The archive is specific that the health arrives
    /// **when it returns**, which is the whole risk of the ability: the cut
    /// lands immediately and the payment has to survive the flight home.
    pub banked: i32,
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
            banked: 0,
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

    /// What it deals each time it connects.
    pub fn damage(&self) -> i32 {
        self.kind.damage(&self.source())
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
    /// **A tornado is always fully grown.** It was cut loose from a pillar
    /// that had already finished growing in the sense that matters -- it is
    /// leaving, not arriving -- so its own `progress` (which now measures how
    /// close it is to burning out, not how established it is) has nothing to
    /// do with its size. Shrinking a tornado toward nothing as its life ran
    /// out would read as it fading, and it does not fade; it moves until its
    /// clock or the arena stops it.
    pub fn pillar_volumes(&self) -> (Pillar, Pillar) {
        let grown = if self.kind == EffectKind::FireTornado {
            Fx::ONE
        } else {
            self.progress()
        };
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
    /// was cut loose from stood, `dir` is the line Cataclysm was aimed along,
    /// and how far past `pos` it has arrived is `age` frames of
    /// `tornado_speed`. A rollback re-simulating the middle of its flight
    /// computes the same point every time it asks, rather than integrating
    /// toward it one frame at a time and landing somewhere near. Meaningless
    /// -- and never called -- on anything but a [`EffectKind::FireTornado`].
    pub fn tornado_pos(&self) -> V3 {
        let travelled = t::tornado_speed()
            .mul(Fx::from_int(self.age as i32))
            .mul(DT);
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
            radius: t::spike_radius(),
            bottom: Fx::ZERO,
            top: t::spike_height(),
        }
    }

    /// Radius of a field effect. Drain fields do not grow; they are a place.
    pub fn field_radius(&self) -> Fx {
        match self.kind {
            EffectKind::BlackSpike => t::spike_radius(),
            EffectKind::FirePillar | EffectKind::FireTornado => self.pillar_volumes().0.radius,
            EffectKind::Bloodletter => t::bloodletter_radius(),
            EffectKind::Grasp => t::grasp_arm_radius(),
            EffectKind::GuillotineLotus => t::lotus_blade_radius(),
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

    /// Where the blade is, as a fraction of its throw: out to one at the turn
    /// and back to zero at the catch.
    ///
    /// A triangle rather than a curve on purpose. A thrown blade that eased in
    /// and out would hang at the far end, and hanging is what a *placed* effect
    /// does; this one is meant to read as a single continuous throw whose only
    /// event is the turn.
    fn out_and_back(&self) -> Fx {
        let p = self.progress();
        let half = Fx::ratio(1, 2);
        if p.raw() <= half.raw() {
            p.mul(Fx::from_int(2))
        } else {
            Fx::ONE.sub(p).mul(Fx::from_int(2))
        }
    }

    /// True once the blade has turned and is on its way home.
    pub fn returning(&self) -> bool {
        self.age as u32 * 2 > self.life as u32
    }

    /// Which pass the blade is on: nought out, one back.
    pub fn pass(&self) -> usize {
        self.returning() as usize
    }

    /// Where the blade is this frame.
    pub fn blade_at(&self) -> V3 {
        self.pos
            .add(self.dir.scale(self.reach.mul(self.out_and_back())))
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
        let erupt = t::lotus_erupt().max(1);
        let hold = t::lotus_hold();
        let back = t::lotus_return().max(1);
        let age = self.age;
        if age < erupt {
            LotusPhase::Erupting(Fx::ratio(age as i32, erupt as i32))
        } else if age < erupt + hold {
            LotusPhase::Held
        } else {
            let through = age - erupt - hold;
            LotusPhase::Returning(Fx::ratio(through.min(back) as i32, back as i32))
        }
    }

    /// How far out the blades are, as a fraction of `lotus_radius`.
    ///
    /// Out on a curve that is fastest at the start, because the eruption is
    /// meant to be over before the victim can answer it; home on the mirror of
    /// that, which is what "slowing" means -- the blades come off full speed
    /// and settle into the shadow rather than snapping back into it.
    fn lotus_extension(&self) -> Fx {
        match self.lotus_phase() {
            // Full speed on the first frame, arriving at rest.
            LotusPhase::Erupting(p) => crate::math::ease_out(p),
            LotusPhase::Held => Fx::ONE,
            // The same curve run backwards: off the mark at speed, and
            // decelerating into the shadow. That is the "slowing" in the
            // ability's own description of itself.
            LotusPhase::Returning(p) => Fx::ONE.sub(crate::math::ease_out(p)),
        }
    }

    /// How far out the blades were on the frame before this one.
    fn lotus_extension_before(&self) -> Fx {
        let mut before = *self;
        before.age = self.age.saturating_sub(1);
        before.lotus_extension()
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
            self.lotus_head(blade, centre, self.lotus_extension_before()),
            self.lotus_at(blade, centre),
        )
    }

    /// Where one blade is this frame, around a centre the caller supplies.
    ///
    /// **The centre is passed in rather than read from `pos`** because it is
    /// the shadow's live position, and the shadow moves: recall it with the
    /// blades out and the six of them are dragged across the arena behind it,
    /// which is most of what the ability is for.
    ///
    /// Each blade leaves on its own sixth of the circle and keeps turning as it
    /// goes -- `lotus_curl` turns over the full extension -- so the six of them
    /// open like petals rather than as spokes of a wheel. They rise on the way
    /// out and come down on the way in, on the same fraction, so the arc is one
    /// motion rather than a height bolted onto a radius.
    pub fn lotus_at(&self, blade: usize, centre: V3) -> V3 {
        self.lotus_head(blade, centre, self.lotus_extension())
    }

    /// One blade's head at a given extension. The shape of the flower, with the
    /// clock taken out of it so the sweep above can ask for two frames at once.
    fn lotus_head(&self, blade: usize, centre: V3, out: Fx) -> V3 {
        let bearing = Fx::ratio(blade.min(LOTUS_BLADES - 1) as i32, LOTUS_BLADES as i32)
            .add(t::lotus_curl().mul(out));
        let along = V3::from_turns(bearing);
        let reach = t::lotus_radius().mul(out);
        V3::new(
            centre.x.add(along.x.mul(reach)),
            centre.y.add(t::lotus_rise().mul(crate::math::arch(out))),
            centre.z.add(along.z.mul(reach)),
        )
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

    fn bit(part: usize, victim: usize) -> u32 {
        1u32 << (part * VICTIMS + victim)
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

    /// Forget everyone hit so far, so the next pass starts clean.
    pub fn forget_hits(&mut self) {
        self.struck = 0;
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
