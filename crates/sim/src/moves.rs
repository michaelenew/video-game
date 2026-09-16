//! Move tables.
//!
//! This is the tuning surface for combat. Frame counts here decide the entire
//! neutral game, so the derived numbers below -- on-block and on-hit advantage
//! -- matter more than the raw ones. A move that is plus on block beats a move
//! that is minus, regardless of how the damage compares.
//!
//! `cargo run -p sim --bin frametable` prints all of it.

use crate::class::Class;
use crate::fixed::Fx;
use crate::math::V3;

#[derive(Clone, Copy, Debug)]
pub struct Move {
    pub name: &'static str,
    /// Frames before the hitbox exists. Under about 15 this is unreactable and
    /// must be *anticipated*; above that it can be answered on sight.
    pub startup: u16,
    /// Frames the hitbox is live.
    pub active: u16,
    /// Frames after, during which you cannot act. This is the cost.
    pub recovery: u16,
    pub damage: i32,
    pub reach: Fx,
    pub radius: Fx,
    pub hitstun: u16,
    pub blockstun: u16,
    pub knockback: Fx,
    /// Goes through guard. The answer to a turtle.
    pub unblockable: bool,
    /// False means an overhead: a crouching opponent ducks it.
    pub hits_crouching: bool,
    /// Requires the class mechanic to be in a particular state -- shield in
    /// hand, shadow placed, meter deep enough. Enforced per class.
    pub needs_mechanic: bool,
    /// Frames this move suspends your fall when thrown in the air.
    ///
    /// Per move rather than universal, because the hang *is* the move's air
    /// identity: a rising strike that holds you up for a beat plays completely
    /// differently from one that drops you through it, and both are worth
    /// having. Zero means gravity never stops.
    pub air_stall: u16,
    /// Vertical speed given to whoever this hits. Positive takes them off the
    /// ground with an uppercut; **negative spikes them into it**, which is the
    /// Champion's aerial hammer and the reason this number is signed.
    pub launch: Fx,
    /// Upward speed the *attacker* gains when the move starts. A leaping move
    /// commits you to the air along with your victim.
    pub self_lift: Fx,
    /// On hit, hold the victim for this many frames at arm's length. Zero is a
    /// normal hit; anything else is a grab.
    pub grabs: u16,
    /// Which persistent effect this move leaves behind, if any.
    /// See `effects::EffectKind::from_code`.
    pub effect: u8,
    /// Which of the three lines of effect this move uses, as
    /// [`crate::aim::Kind`]'s own numbering.
    ///
    /// **Declared, not inferred.** It used to be worked out from what the move
    /// leaves behind, which answered for the two abilities that plant something
    /// and quietly called everything else a swing -- so Fissure, which the kit
    /// describes as racing along the ground to a point, came out as a bubble
    /// seven metres in front of the body. Every move states its own.
    pub aim_code: u8,
    /// Percent of walking speed you keep while the move runs.
    ///
    /// A poke slows you; a committed move slows you to a crawl. **Nothing is
    /// zero.** Zero roots you, and rooting was what commitment used to mean --
    /// but a character who ignores the stick reads as the game snatching the
    /// controls away, which was the complaint that took it off the pokes first
    /// and off the heavies after. What commitment means is that you cannot
    /// jump, dodge, guard or throw anything else until the move is finished,
    /// and none of that is this number: see `crate::state::Action::actionable`.
    ///
    /// See [`tuning::poke_mobility`] and [`tuning::committed_mobility`] for the
    /// two reference speeds, and `docs/design/controls.md` for the table they
    /// sit in. The palette can still set this to zero; the feel harness is what
    /// says it should not be.
    ///
    /// [`tuning::poke_mobility`]: crate::tuning::poke_mobility
    /// [`tuning::committed_mobility`]: crate::tuning::committed_mobility
    pub mobility: u8,
    /// How far a swing travels, in turns, and **which way**.
    ///
    /// Signed: a positive arc runs from the `+` end of its span to the `-` end,
    /// and a negative one reverses -- which is what makes a rising cut a rising
    /// cut rather than a falling one from the same two numbers. Zero is a move
    /// whose shape does not sweep.
    ///
    /// Which way the `+` end *is* depends on the plane, and it is worth being
    /// exact about because the answer is not the same for both:
    ///
    /// * [`Plane::Upright`] and [`Plane::Diagonal`]: positive **comes down**.
    ///   The head starts at the top of the arc and finishes at the bottom.
    /// * [`Plane::Flat`]: positive starts on the fighter's **left** and sweeps
    ///   across to the right. The body is authored in a left-handed frame
    ///   dropped into a right-handed world (see [`crate::aim::across`]), so
    ///   which of the two a positive rotation about the vertical actually
    ///   produces is a fact to read off rather than to reason about -- this
    ///   sentence used to say the opposite, and the Champion's spinning finisher
    ///   is what found it out: the clip swept one way and the volume the other.
    ///   `view/tests/kinematics.rs` is what keeps that honest now.
    pub arc: Fx,
    /// Frames between one connection and the next for a move that keeps
    /// hitting. Zero is the normal rule: a swing lands once.
    ///
    /// It exists for exactly one move -- the Champion's Rush slash, which is
    /// meant to be run *through* a crowd -- and it is the one place where the
    /// on-hit arithmetic below stops meaning anything, because the move is
    /// still in its active frames when the next cut lands.
    pub rehit: u16,
    /// Frames this move can be wound up for while its button is held. Zero is
    /// the normal rule: a press throws the move.
    ///
    /// What the hold *buys* is the move's business. The Grasp -- the only one
    /// that channels -- buys reach with it, walking from [`channel_from`] out
    /// to [`reach`], so a player who wants the arms to close ten metres away
    /// has to stand still and hold the button while they do.
    ///
    /// [`channel_from`]: Move::channel_from
    /// [`reach`]: Move::reach
    pub channel: u16,
    /// Where a channelled move reaches on a hold of nothing at all -- the near
    /// end of the slider `reach` is the far end of. Meaningless when
    /// [`channel`](Move::channel) is zero.
    pub channel_from: Fx,
    /// Health the caster pays the moment the move starts.
    ///
    /// Zero on almost everything. It is the Blood mage's whole economy -- see
    /// `docs/design/kits/blood-mage.md` -- and it is a move field rather than a
    /// class rule because the *spread* is the design: an auto you can throw all
    /// day costs a trickle, and the committed casts cost real blood.
    ///
    /// **It can never kill you.** Self-damage clamps at one, the same rule the
    /// Dual mage's meter burn already follows: dying to your own button is not
    /// a decision anybody made.
    pub cost: i32,
    /// Percent of the damage this move deals that comes back as health.
    ///
    /// The other half of the same economy: the cost is paid on the press and
    /// the return is earned on the hit, so missing is the punishment. Applies
    /// to what the move itself deals and to what anything it leaves behind
    /// drains -- one number per ability, wherever the damage happens to land.
    pub leech: u8,
    /// This move's share of the shared repeat lockout, as a percentage.
    ///
    /// 100 on everything until somebody plays it and finds otherwise, which is
    /// the point of it being a column rather than a constant: the shared number
    /// is the rule and this is where a move argues with it. Zero exempts a move
    /// from the rule outright.
    ///
    /// The frames themselves are [`Move::repeat_lock`]; this is only the
    /// multiplier, because the number it multiplies is tuned in one place for
    /// the whole roster.
    pub repeat_mul: u8,
    /// The shortest gap between one activation of this ability and the next,
    /// for the ones that are used more than once per cast.
    ///
    /// Zero on everything, including the two that have a second activation:
    /// today neither of them is gated and the number is here to be found by
    /// playing rather than guessed at a desk. Meaningless on a move whose own
    /// button does not reactivate it -- see [`reactivates`].
    pub reactivate: u16,
    /// How far the move itself carries the body forward, in metres.
    ///
    /// **A step is not mobility.** [`mobility`] is how much of your own walk
    /// you keep while a move runs -- you steering, slowly. This is the move
    /// steering you: a distance it covers along the facing it locked, whether
    /// or not the stick is touched, spent over the frames that end with the
    /// last active one so that the body arrives with the weapon. The two add,
    /// which is what lets a cut thrown while strafing come out diagonally
    /// instead of snapping to the front.
    ///
    /// Zero on almost everything, and that is the point of it being a column:
    /// a move that does not step says so, and a move that does says how far.
    /// Signed, so a swing can also give ground -- nothing does yet.
    ///
    /// **It is the distance covered by the time the weapon lands**, which is the
    /// number that decides spacing. The window is [`tuning::step_lead`] frames of
    /// run-up and then the frame the hitbox appears on, shared across the whole
    /// roster, so the *shape* of a step is a rule and only its size is a per-move
    /// decision: a long distance over that fixed window is a dash and a short one
    /// is a step, which is why one number covers both.
    ///
    /// The body keeps travelling after that, because the momentum is still under
    /// it and the hindrance ramp spends it over a few frames. That is the
    /// follow-through rather than the step, and it is what stops a lunge ending
    /// like a wall.
    ///
    /// [`mobility`]: Move::mobility
    /// [`tuning::step_lead`]: crate::tuning::step_lead
    pub step: Fx,
    /// The volume this move puts in the world. See [`Shape`].
    pub shape: Shape,
    /// Which arm it comes out of. See [`crate::aim::Hand`].
    pub hand: crate::aim::Hand,
}

/// What an attack's hit volume looks like.
///
/// Until the Champion was rebuilt every move in the game was a `Cylinder`: a
/// disc at arm's length, live for its whole active window, with no vertical
/// extent and nothing to distinguish a spear thrust from a hammer swing but
/// two numbers. That is still the rule for every class that has not been given
/// shapes yet, and it is stated here as a shape rather than left implicit so
/// that "this move has not been looked at" is visible in the table.
///
/// The others are **capsules**: a line from the hand to the head of the
/// weapon, with `Move::radius` as its thickness, moving over the active frames
/// the way the weapon does. A capsule is the smallest thing that can tell a
/// sweep from a thrust, and -- unlike the disc -- it has a top and a bottom, so
/// an attack thrown from the air can miss someone standing under it.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Shape {
    /// No hit volume at all. A move that is pure movement -- the pole vault --
    /// rather than one that happens to miss.
    None,
    /// The original rule: a disc of `radius` at `reach` along the facing,
    /// tested flat, with no top and no bottom.
    Cylinder,
    /// A swing. The weapon is a line from the body out to `reach`, and its head
    /// travels through `arc` turns over the active window.
    Swing(Plane),
    /// A thrust. The weapon is a line along the aim that extends to `reach`
    /// over the active window and does not travel sideways.
    Thrust,
    /// A wing: **a thin curved blade, swept on a ring the caster is not in the
    /// middle of.**
    ///
    /// ```text
    ///        .-  -  -.                 a section of a torus, in the plane of
    ///     .'          '.               the floor. Inner arc at
    ///    ;      (x)     :              `tuning::wing_inner` of the reach,
    ///     '.          .'               outer arc at `Move::reach`, and
    ///        ' - , - '                 nothing in between
    ///          start
    ///                       (x) the ring's middle: `tuning::wing_offside`
    ///   o                       toward the caster's *other* arm and
    ///   |   the caster          `tuning::wing_ahead` in front of them
    /// ```
    ///
    /// It starts behind the caster on the punching arm's own side and finishes
    /// `tuning::wing_finish` off their centre line, in front of that arm's own
    /// hand -- not dead ahead, because which of two mirrored autos just landed
    /// is a thing the player reads off where it landed.
    ///
    /// The volume out on any one frame is the section's own **radius** -- a
    /// line from the inner arc to the outer one. That is not an approximation
    /// of a curved shape: a radius of an annulus is straight, and it is the
    /// whole of the section at that angle. The ring is what the sweep carves
    /// over the active window, which is the thing the player sees.
    ///
    /// Three things separate it from a [`Shape::Swing`], and they are why it is
    /// its own shape rather than a swing with unusual numbers. It is hung off a
    /// **ring** rather than off a shoulder, so it curves rather than reaches. It
    /// has a **hole**, and a big one: the band is the outer quarter of the ring
    /// and there is no haft to stand inside. And it **starts behind the caster**
    /// rather than travelling across the front of her -- the punch throws it and
    /// it overtakes the punch.
    ///
    /// Which way round it sweeps comes from [`Move::hand`], so the two mirrored
    /// autos share one `arc` -- see [`crate::aim::Hand::outward`]. For this
    /// shape `arc` is **how far back the section starts**, measured from where
    /// it finishes, rather than a span centred on the facing.
    ///
    /// The geometry is [`crate::moves::wing`], and the last active frame is not
    /// a section at all: see [`Wing::tip`].
    Wing,
}

impl Shape {
    /// Whether this move can connect with anybody at all.
    pub const fn strikes(self) -> bool {
        !matches!(self, Shape::None)
    }

    /// Whether the old flat rule applies -- no vertical extent, and the
    /// defender's height is not consulted.
    pub const fn flat(self) -> bool {
        matches!(self, Shape::Cylinder)
    }
}

/// Which way a swing sweeps.
///
/// All three are the same rotation about a different axis, and every axis is
/// square to the aim -- so the head of the weapon always travels through the
/// aim, and what the plane chooses is *which way round it goes*. [`Flat`] turns
/// about the vertical and [`Upright`] about the horizontal; [`Diagonal`] is the
/// continuum between them, stopped at one tilt.
///
/// [`Flat`]: Plane::Flat
/// [`Upright`]: Plane::Upright
/// [`Diagonal`]: Plane::Diagonal
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Plane {
    /// Across the body, turning about the vertical. A cut that catches
    /// everything in front of you and nothing above or below it.
    Flat,
    /// Down the body, turning in the vertical plane that contains the aim. A
    /// slam, a rising cut, and everything the Champion does in the air.
    Upright,
    /// **Corner to corner**: the upright plane rolled off the vertical by
    /// [`tuning::cut_roll`], so the head starts high over one shoulder and
    /// finishes low past the other hip.
    ///
    /// ```text
    ///     Diagonal(Right)        Diagonal(Left)          Upright
    ///        \                        /                     |
    ///         \      o               /    o                  |  o
    ///          \    /|\             /    /|\                 | /|\
    ///           \                  /                         |
    /// ```
    ///
    /// The [`Hand`] is **which shoulder it starts over**, and that is the whole
    /// of its sign: one tuned tilt serves both of the sword's mirrored cuts,
    /// exactly the way one tuned span serves the Dual mage's two wings. See
    /// [`crate::aim::Hand::outward`].
    ///
    /// It is a different question from [`Move::hand`], which is the shoulder the
    /// volume *leaves from*. A two-handed sword is gripped at the sternum and
    /// still swung from over the right shoulder, so the two are a centre-line
    /// origin and a right-handed plane, and collapsing them would be a lie about
    /// one or the other.
    ///
    /// Why it exists at all: a cut that is either level or vertical has to pick
    /// between owning the width of the front and owning the height of a body,
    /// and a real sword cut owns both because it is thrown across the diagonal
    /// of the target. The flat version passes over anything crouched and the
    /// upright version misses anything that stepped aside; this one is what a
    /// swordsman actually does.
    ///
    /// [`tuning::cut_roll`]: crate::tuning::cut_roll
    /// [`Hand`]: crate::aim::Hand
    Diagonal(crate::aim::Hand),
}

impl Move {
    /// Which of the three kinds of aiming this move uses.
    ///
    /// Every move has an answer, and the answer is here rather than at each
    /// call site -- a caller that decided for itself is how the crosshair and
    /// the ability came to disagree in the first place. See [`crate::aim`],
    /// which is the only place allowed to act on it.
    pub fn aim(&self) -> crate::aim::Kind {
        crate::aim::Kind::from_code(self.aim_code)
    }

    /// Frames the attacker is still busy after the first active frame connects.
    pub const fn busy_after_contact(&self) -> i32 {
        (self.active as i32 - 1) + self.recovery as i32
    }

    /// Frame advantage on block. **Negative is punishable**, which most moves
    /// should be -- that is what makes blocking worth doing.
    pub const fn on_block(&self) -> i32 {
        self.blockstun as i32 - self.busy_after_contact()
    }

    /// Frame advantage on hit. Positive means you keep the initiative.
    ///
    /// Meaningless for a move that re-hits: the arithmetic assumes the
    /// exchange is over once it connects, and a move that connects again four
    /// frames later has not finished the exchange.
    pub const fn on_hit(&self) -> i32 {
        self.hitstun as i32 - self.busy_after_contact()
    }

    /// Total commitment if it whiffs entirely.
    pub const fn whiff_cost(&self) -> u16 {
        self.startup + self.active + self.recovery
    }

    /// How long after throwing this move you may not throw *it* again.
    ///
    /// The shared number from [`tuning::repeat_lockout`] scaled by this move's
    /// own [`repeat_mul`]. See that function for why this is not a cooldown.
    ///
    /// [`tuning::repeat_lockout`]: crate::tuning::repeat_lockout
    /// [`repeat_mul`]: Move::repeat_mul
    pub fn repeat_lock(&self) -> u16 {
        (crate::tuning::repeat_lockout() as u32 * self.repeat_mul as u32 / 100).min(u16::MAX as u32)
            as u16
    }

    /// Frames the lockout keeps you waiting *beyond* the move itself.
    ///
    /// The number that says whether the rule touches this move at all. A move
    /// that commits you for longer than its own lockout comes out of recovery
    /// with the lockout already expired, so this is zero and the rule is
    /// invisible on it -- which is every committed heavy in the game. What is
    /// left is the autos and the fast pokes, which is the set the rule is for.
    pub fn repeat_idle(&self) -> u16 {
        self.repeat_lock().saturating_sub(self.whiff_cost())
    }

    /// Whether this move strikes on its own, or only places something -- or
    /// nothing at all.
    ///
    /// Two ways to answer no, and they are different sentences.
    ///
    /// A radius of zero is not a tiny hitbox, it is *no* hitbox: the move is a
    /// gesture that puts something into the world and the thing it put there
    /// does all of the hitting. The Blood mage's auto and her Grasp are both
    /// that shape -- the blade and the arms carry the damage, and the caster's
    /// own body never touches anybody.
    ///
    /// [`Shape::None`] is the other: a move that puts nothing in the world
    /// either, because it is pure movement. The Champion's pole vault is the
    /// only one.
    pub const fn strikes(&self) -> bool {
        self.shape.strikes() && self.radius.raw() > 0
    }

    /// Is this move wound up while its button is held?
    pub const fn channels(&self) -> bool {
        self.channel > 0
    }

    /// How far out this move reaches after being held for `held` frames.
    ///
    /// Between this move's near knob and its own `reach`, linearly, so the two
    /// are the ends of one slider and the hold is what walks between them. A
    /// move that does not channel is always at its full reach, which is what
    /// makes this safe to ask of any move.
    ///
    /// **The hold is the whole of the answer.** It does not ask what the
    /// crosshair is pointed at: the aim picks a *direction* and this picks a
    /// *distance*, and keeping the two apart is what makes the wind-up mean one
    /// thing. A Grasp held to full range at a wall six metres away reaches ten
    /// metres, through the wall -- the wall is not a shorter Grasp.
    pub fn reach_after(&self, held: u16) -> Fx {
        if !self.channels() {
            return self.reach;
        }
        let near = self.channel_from.min(self.reach);
        let at = Fx::ratio(held.min(self.channel) as i32, self.channel as i32);
        near.add(self.reach.sub(near).mul(at))
    }

    /// Health returned for `dealt` damage, rounded down.
    pub const fn leeched(&self, dealt: i32) -> i32 {
        if self.leech == 0 || dealt <= 0 {
            return 0;
        }
        (dealt * self.leech as i32) / 100
    }
}

// ---------------------------------------------------------------------------
// The roster
// ---------------------------------------------------------------------------
//
// Names and prose only. **The numbers live in the Oven** (`crates/sim/oven.rs`,
// baked to `tuned.rs`), because they are edited in the running game and written
// back from there -- keeping a second copy here would guarantee the two drift,
// and a move table that disagrees with the game is worse than no table.
//
// `cargo run -p sim --bin frametable` prints the current values.

/// Every class's moves, in slot order: poke, committed, special, and then
/// whatever else that class has.
///
/// **The list is a different length for each class, and that is the design.**
/// Three is the shared vocabulary. The Blood mage has a fourth on `E`, because
/// her mechanic is *health* -- not a thing you press a button to change -- so
/// the key is free for an ability, and an ability needs a startup, a reach and
/// a cost like any other. The Champion has nineteen, because its three mouse
/// buttons are three weapons, each of them chains three hits deep, and each of
/// them behaves differently on foot, in the air, mid-Rush and on the way off
/// the floor: see [`champion`].
///
/// Storage is packed to these counts, so a class that does not have a slot does
/// not have knobs for one either -- see [`slots`] and [`bound`].
const NAMES: [&[&str]; 6] = [
    // Bulwark -- committed, not slow. Wins by denying space.
    //   Bash: fast poke, slightly minus on block so it is not a free mash.
    //   Slam: the overhead. Heavily punishable if read, heavily rewarding if not.
    //   Grapple: beats guard outright, loses badly to dodge.
    &["Bash", "Slam", "Grapple"],
    // Champion -- three weapons on three buttons, and the row of the grid is
    // the situation your feet are in. Nineteen moves: see `champion`.
    //
    // The first nine are the **chain**, three hits deep, and every hit is a
    // free choice of weapon -- that is the class fantasy stated as a move list.
    // Reading them as three rows of three rather than as three combos is the
    // point: hit one is any of the first three, hit two is any of the second
    // three, hit three is any of the last, and nothing says the three have to
    // be the same weapon.
    &[
        "Sword",
        "Hammer",
        "Spear",
        "Backcut",
        "Uproot",
        "Skewer",
        "Upcut",
        "Earthbreaker",
        "Whirl",
        "Air sword",
        "Air hammer",
        "Air spear",
        "Rush slash",
        "Rush sweep",
        "Rush stab",
        "Rising cut",
        "Uppercut",
        "Pole drive",
        "Pole vault",
    ],
    // Shadow Reaver -- two bodies. Options are a function of the line between
    // them, and the line always exists: the shadow is never nowhere.
    //
    // The two buttons are swapped against every other class, and the reason is
    // the sixth sentence of the grammar rather than an exception to it: **the
    // mouse means where.** Sending the shadow is the one thing in this kit the
    // crosshair aims, so it is on the mouse; Executioner is a swing off the
    // body and does not care, so it is on the key.
    //   Slash: the auto, on left click.
    //   Executioner: the committed melee, on `E` as well as shift+left.
    //   Guillotine: six blades erupt from the shadow and come back to it, so it
    //     is aimed at the mechanic and the mechanic does all the hitting.
    //   Send shadow: on right click, and a real move rather than a state flip.
    //     It throws the second body out fast and, pressed again, dashes it home
    //     through anybody in the way.
    &["Slash", "Executioner", "Guillotine", "Send shadow"],
    // Elementalist -- terrain author. Ranged, and creates its own targets.
    // Seven: four on the ground, and a whole row of three off it. See
    // [`elementalist`].
    //   Bolt: the game's one *skillshot* -- an instant line from her hand to
    //   whatever the crosshair is on, so its `reach` is the max-range sphere
    //   and its `radius` is the line's thickness. Resolved where it is fired
    //   rather than by the hitbox loop, and its hitstun and knockback are zero
    //   on purpose: it takes the charge off whoever it catches and gives them
    //   their frames straight back. See `crate::bolt`.
    //   Cataclysm: the heavy, on right click. A long wind-up and then the same
    //   kind of instant line Bolt throws, resolved the same way and for the
    //   same reason -- what it does depends on what it meets first, and none
    //   of that fits a hitbox that lives for a few frames in front of her body.
    //   A structure in the way is destroyed outright rather than kicked, and
    //   breaks into thrown debris (see `crate::debris`); a fire pillar in the
    //   way is not charged, it is torn loose into a travelling fire tornado
    //   (see `crate::effects::EffectKind::FireTornado`).
    //   Air bolt, Gale, Landfall: the same three buttons with her feet off the
    //   floor. Both shots *travel*, unlike anything she throws standing up --
    //   see `crate::gust` -- and neither has a hitbox of its own, which is
    //   what `Shape::None` in [`shape`] says. Landfall does: a disc on the
    //   floor where she arrives.
    &[
        "Bolt",
        "Fissure",
        "Fire pillar",
        "Cataclysm",
        "Air bolt",
        "Gale",
        "Landfall",
    ],
    // Blood mage -- sustain through aggression. Everything costs health, and
    // every one of these has a cost in the table to prove it.
    //   Bloodletter: the auto. Out to a fixed distance and back, cutting on
    //     both passes, and the blood it takes comes home with it.
    //   Rend: the committed poke it was before the auto took its slot.
    //   Grasp: four arms out in a cone that arc back inward to meet. Caught by
    //     all four and you are rooted.
    //   Black spike: on `E`, because the class has no other use for the key.
    &["Bloodletter", "Rend", "Grasp", "Black spike"],
    // Dual mage -- melee mage riding between two forces, one in each arm. Six
    // moves on five inputs, and the two on the bare clicks are the class: see
    // [`dual`].
    //   Dark auto / Light auto: the autos, and the steering wheel. Left is
    //     dark, right is light, and each one is a punch that opens into a wing.
    //     One pulls and one pushes, so which arm you throw is a spacing
    //     decision as well as a meter one.
    //   Light lance / Dark lance: both on middle click, which has no side --
    //     so the form is the force she is carrying. Light detonates at the far
    //     end of the line; dark tethers what it catches and drains it.
    //   Sweep: on `E`, because the meter is steered by which button attacks
    //     rather than by a key of its own, so the mechanic key is free.
    //   Judgement: the finisher. No depth gate -- depth is what makes it big.
    &[
        "Dark auto",
        "Light lance",
        "Judgement",
        "Sweep",
        "Light auto",
        "Dark lance",
    ],
];

/// The slots every class has: poke, committed, special. Two classes have more.
pub const SLOTS: usize = 3;

// ---------------------------------------------------------------------------
// The Champion's nineteen
// ---------------------------------------------------------------------------

/// The Champion's move list, as a **grid**: a row per situation, a column per
/// weapon, and the first three rows are one three-hit chain rather than three.
///
/// This is the whole of the class's input scheme and it is worth reading as a
/// table rather than as a list of nineteen moves:
///
/// ```text
///                     left click      middle click    right click
///   on foot, hit 1    Sword           Hammer          Spear
///   on foot, hit 2    Backcut         Uproot          Skewer
///   on foot, hit 3    Upcut           Earthbreaker    Whirl
///   in the air        Air sword       Air hammer      Air spear
///   rushing           Rush slash      Rush sweep      Rush stab
///   leaving the floor Rising cut      Uppercut        Pole drive
///   (rushing, aimed at the floor: Pole vault)
/// ```
///
/// The button is the **weapon** and never changes meaning; the row is the
/// situation. That is the entire thing a new player has to learn, and it is why
/// the Champion can carry nineteen moves on three buttons without a single
/// modifier: you never choose a move, you choose a weapon, and the situation
/// chooses the move.
///
/// **The first three rows are one chain.** Connect with any of the first row
/// and the same three buttons throw the second row; connect again and they
/// throw the third. Which weapon each hit is thrown with is a free choice every
/// time, so sword into spear into hammer is an ordinary thing to do and is the
/// class fantasy -- one haft, three heads, chosen a hit at a time. See
/// [`link_of`] and `state::champion_move`.
///
/// The last row is the **takeoff**: a weapon thrown on the same press as jump,
/// which is how this class leaves the ground with something already swinging.
///
/// The nineteenth is the Pole vault, which shares right click with the Rush
/// stab and is separated by where you are pointing: a spear planted in the
/// ground vaults, a spear levelled at someone stabs.
pub mod champion {
    /// Weapons, in button order. The column of the grid.
    pub const SWORD: u8 = 0;
    pub const HAMMER: u8 = 1;
    pub const SPEAR: u8 = 2;

    /// Rows. The situation, as a base to add a weapon to.
    ///
    /// The first three are the chain, in order, which is what lets the stage a
    /// player has reached be a number rather than a table: the move is
    /// `LINKS[stage] + weapon`.
    pub const FIRST: u8 = 0;
    pub const SECOND: u8 = 3;
    pub const THIRD: u8 = 6;
    pub const IN_THE_AIR: u8 = 9;
    pub const RUSHING: u8 = 12;
    pub const TAKEOFF: u8 = 15;

    /// The three rows of the chain, deepest last.
    pub const LINKS: [u8; 3] = [FIRST, SECOND, THIRD];
    /// How many hits deep the chain goes.
    pub const DEPTH: u8 = LINKS.len() as u8;

    pub const SWORD_GROUND: u8 = FIRST + SWORD;
    pub const HAMMER_GROUND: u8 = FIRST + HAMMER;
    pub const SPEAR_GROUND: u8 = FIRST + SPEAR;
    pub const BACKCUT: u8 = SECOND + SWORD;
    pub const UPROOT: u8 = SECOND + HAMMER;
    pub const SKEWER: u8 = SECOND + SPEAR;
    pub const UPCUT: u8 = THIRD + SWORD;
    pub const EARTHBREAKER: u8 = THIRD + HAMMER;
    pub const WHIRL: u8 = THIRD + SPEAR;
    pub const AIR_SWORD: u8 = IN_THE_AIR + SWORD;
    pub const AIR_HAMMER: u8 = IN_THE_AIR + HAMMER;
    pub const AIR_SPEAR: u8 = IN_THE_AIR + SPEAR;
    pub const RUSH_SLASH: u8 = RUSHING + SWORD;
    pub const RUSH_SWEEP: u8 = RUSHING + HAMMER;
    pub const RUSH_STAB: u8 = RUSHING + SPEAR;
    pub const RISING_CUT: u8 = TAKEOFF + SWORD;
    pub const UPPERCUT: u8 = TAKEOFF + HAMMER;
    pub const POLE_DRIVE: u8 = TAKEOFF + SPEAR;
    /// The odd one out: right click during a Rush, aimed at the floor.
    pub const POLE_VAULT: u8 = 18;

    pub const COUNT: usize = 19;

    /// Which weapon a move is thrown with. The column of the grid, except for
    /// the vault, which is planted with the spear like everything else on
    /// right click.
    pub const fn weapon(kind: u8) -> u8 {
        if kind == POLE_VAULT { SPEAR } else { kind % 3 }
    }

    /// How many hits into the chain this move is, if it is one of the nine.
    ///
    /// `Some(0)` is an opener, `Some(2)` a finisher, `None` a move that is not
    /// part of the chain at all -- an aerial, a Rush move, a takeoff. Asked
    /// rather than inferred from the index anywhere else, because "is this
    /// thing a chain link" is the question three separate rules need answered
    /// and each of them getting it from arithmetic is how one of them ends up
    /// disagreeing.
    pub const fn link_of(kind: u8) -> Option<u8> {
        if kind < IN_THE_AIR {
            Some(kind / 3)
        } else {
            None
        }
    }

    /// The move a given weapon throws at a given depth into the chain.
    ///
    /// Clamped rather than wrapped: a fourth press is the finisher again in
    /// arithmetic and a fresh opener in play, and it is the caller -- which
    /// knows whether the chain is still alive -- that decides which. Clamping
    /// keeps a bad stage from indexing off the end of the table.
    pub const fn link(stage: u8, weapon: u8) -> u8 {
        let stage = if stage >= DEPTH { DEPTH - 1 } else { stage };
        LINKS[stage as usize] + weapon
    }

    /// Is this move thrown as the feet leave the floor?
    pub const fn is_takeoff(kind: u8) -> bool {
        kind >= TAKEOFF && kind < POLE_VAULT
    }
}

// ---------------------------------------------------------------------------
// The Dual mage's six
// ---------------------------------------------------------------------------

/// The Dual mage's move list, and **which way each one pushes the meter**.
///
/// The class holds two forces apart, one in each arm, and the whole of its
/// mechanic is that *which button you attacked with* decides which way you
/// drift -- see `docs/design/dual-mage.md`. So the list is worth reading as two
/// columns rather than as five moves:
///
/// ```text
///              darker              lighter        whichever she is carrying
///   click      Dark auto (L)       Light auto (R)  Lance (M) -- and which
///                                                  *form* of it is the force
///                                                  she is carrying
///   key                                           Judgement (Q), Sweep (E)
/// ```
///
/// The two autos are the same punch mirrored: one arm each, one shared set of
/// numbers, and the hand supplies the sign of the arc. Nothing else in the
/// roster is built that way, and it is the reason [`crate::aim::Hand`] exists.
///
/// **Only the autos have a side.** Everything else -- the committed cast, the
/// key abilities -- is made of whichever force she is carrying, which is the
/// last auto that landed, and pushes her further that way. Before the first
/// auto connects she is carrying neither, and a cast pushes her further along
/// whichever way she was already going: the rule the design document states for
/// every input that is neither left nor right.
pub mod dual {
    pub const DARK_AUTO: u8 = 0;
    /// Middle click while she is carrying light: the line flies out and
    /// **detonates at its far end**, so it is a thing you aim *past* somebody.
    pub const LIGHT_LANCE: u8 = 1;
    pub const JUDGEMENT: u8 = 2;
    pub const SWEEP: u8 = 3;
    pub const LIGHT_AUTO: u8 = 4;
    /// Middle click while she is carrying dark: the same line, but it
    /// **tethers** the first thing it hits and drains it until the leash
    /// breaks on distance.
    ///
    /// Slot five rather than slot two, which is the one thing here that is
    /// about storage rather than about the kit: the Oven's move store is packed
    /// in class order with the Dual mage last, so a slot appended to her
    /// leaves every knob index in `tuned.rs` meaning what it was baked with.
    /// Renumbering to put the two Lances side by side would silently rewrite
    /// every number on the class.
    pub const DARK_LANCE: u8 = 5;

    pub const COUNT: usize = 6;

    /// Which force this move throws, if it is one of the two autos.
    ///
    /// **Only the autos have a force of their own.** Everything else takes the
    /// one she is carrying, which is whichever auto landed last -- see
    /// `class::Mechanic::Meter`. That is the mechanic in one sentence: the two
    /// buttons you press constantly decide what everything else is made of.
    ///
    /// Read from the move rather than from the buttons held down, which is the
    /// same reason everything else here is declared: `shift + left click` has
    /// both a modifier and a side in it, and a reader of the input bits has to
    /// know which one wins. The move already knows.
    pub const fn force(kind: u8) -> Option<crate::class::Force> {
        use crate::class::Force;
        match kind {
            DARK_AUTO => Some(Force::Dark),
            LIGHT_AUTO => Some(Force::Light),
            _ => None,
        }
    }

    /// Is this one of the two autos?
    ///
    /// The two things you press constantly, and the only two with a side of
    /// their own. Everything else is made of whichever force they left her
    /// carrying.
    ///
    /// It is also the one exception to depth scaling the *size* of what she
    /// throws -- see `state::depth`. An auto's reach is pinned to the punch
    /// that throws it (`view/tests/kinematics.rs` checks the blade starts where
    /// the fist stops), and a volume that grew away from the animation would
    /// make the one move in the kit thrown every second unreadable. What depth
    /// does to an auto is what it *does*: the damage, and how hard it pulls or
    /// shoves.
    pub const fn is_an_auto(kind: u8) -> bool {
        matches!(kind, DARK_AUTO | LIGHT_AUTO)
    }

    /// Which form of Lance middle click throws, given what she is carrying.
    ///
    /// **The one input in the game that is two moves.** Middle click has no
    /// side, so it cannot pick a direction on the bar -- which is exactly what
    /// makes it the right home for the cast whose *form* is picked by the arm
    /// she last punched with. The two are separate rows in the move table
    /// rather than one row with a flag because the thing that has to differ is
    /// the **wind-up**: a person standing opposite has to be able to tell a
    /// burst they should get out from under from a tether they should break,
    /// and that is frames and a pose, not a damage number.
    pub const fn lance_for(force: crate::class::Force) -> u8 {
        match force {
            crate::class::Force::Light => LIGHT_LANCE,
            crate::class::Force::Dark => DARK_LANCE,
        }
    }

    /// Is this either form of Lance?
    pub const fn is_a_lance(kind: u8) -> bool {
        matches!(kind, LIGHT_LANCE | DARK_LANCE)
    }

    /// Is this the finisher?
    ///
    /// Declared rather than inferred from the binding, for the same reason
    /// everything else on this class is: what makes Judgement the finisher is
    /// that it **throws the bar harder than anything else she has** -- see
    /// `tuning::meter_finisher_push` -- and that is a property of the move, not
    /// of the key it happens to be on.
    pub const fn is_the_finisher(kind: u8) -> bool {
        kind == JUDGEMENT
    }
}

// ---------------------------------------------------------------------------
// The Elementalist's seven
// ---------------------------------------------------------------------------

/// The Elementalist's move list, as **two rows**: what the three buttons do
/// with her feet on the floor, and what they do off it.
///
/// ```text
///                 left click      right click     E
///   standing      Bolt            Cataclysm       Raise -- the mechanic,
///                 (shift: Fissure, Q: Fire pillar)  an instant, no frames
///   in the air    Air bolt        Gale            Landfall
/// ```
///
/// The row is the situation and the button never changes meaning, which is the
/// Champion's grid read one class further: left click is the cheap shot you
/// throw constantly, right click is the committed one, `E` is earth. That is
/// the answer `docs/design/README.md` left open under **Aerials** -- airborne
/// attacks are *variants of their grounded counterparts* rather than a
/// separate move list -- given to the second class.
///
/// **Why air.** Earth is what she is standing on, and off the floor she is not
/// standing on it: the two shots are the element she can reach in the air, and
/// the way back to earth is to go and hit it. That is Landfall, and it is the
/// only one of the three that leaves a structure behind.
///
/// Shift does not modify the air row. There is no airborne Fissure -- a crack
/// racing along the ground is a thing thrown *from* the ground -- so shift plus
/// left click in the air is the Air bolt rather than a move that does not
/// exist.
pub mod elementalist {
    /// Left click, airborne. A bolt of air with a real speed, thrown along the
    /// crosshair -- see `crate::gust`.
    pub const AIR_BOLT: u8 = 4;
    /// Right click, airborne. A disc of air that **grows as it travels**, and
    /// hits harder the bigger it has got: the same inverted spacing Flame
    /// spitter is written around, as a projectile.
    pub const GALE: u8 = 5;
    /// `E`, airborne. The plunge: a long, readable descent that ends in a
    /// stagger on the floor and a slab of rock thrown up at an angle in front
    /// of her. On the ground `E` is still the mechanic and still an instant --
    /// see `moves::on_e`, which answers for the key without knowing where her
    /// feet are, and `state::keyed_move`, which is what does know.
    pub const LANDFALL: u8 = 6;

    pub const COUNT: usize = 7;

    // **No `is_airborne` here, deliberately.** "Which move is this button" is
    // answered once, in `state::elementalist_move` and `state::keyed_move`, and
    // "does this move throw a travelling shot" is answered once, by
    // `gust::Gale::thrown_by`. A third predicate saying the same thing in a
    // third shape is not a declaration, it is a second answer waiting to
    // disagree with the first.
}

/// How many moves a class has.
///
/// Per class rather than a single constant because the Champion legitimately
/// grew: its three mouse buttons are three weapons, the ground chain is three
/// hits deep, and a weapon behaves differently in the air, mid-Rush and on the
/// frame the feet leave the floor -- eighteen moves plus the vault. Every other
/// class still has the original three, and the storage in the Oven is packed to
/// these counts so the five that did not grow cost nothing.
pub const fn slots(class: Class) -> usize {
    match class {
        Class::Champion => champion::COUNT,
        // Seven: the four she throws standing up -- the fourth being
        // Cataclysm on right click, otherwise dead weight on a class with no
        // shield -- and a row of three more for the same buttons with her feet
        // off the floor. See [`elementalist`] and `state::clicked_move`.
        Class::Elementalist => elementalist::COUNT,
        // The fourth is Black spike, on `E`. See `on_e`.
        Class::BloodMage => SLOTS + 1,
        // And Send shadow, on `E`. The Reaver's mechanic *is* a state change,
        // unlike the Blood mage's -- but throwing a second body out across the
        // arena and dashing it back through somebody is not an instant, and a
        // move with a flight, a damage number and a slow needs the same table
        // every other move is in.
        Class::ShadowReaver => SLOTS + 1,
        // Six: an auto on each click, because the two autos are two different
        // moves rather than one move with a modifier; both forms of Lance on
        // middle click, because the form is the force she is carrying rather
        // than the button; and Sweep on `E`. See [`dual`].
        Class::DualMage => dual::COUNT,
        _ => SLOTS,
    }
}

/// Which move the `E` key throws, if the class puts one there.
///
/// `E` is the class mechanic, and for five of the six that is an instant change
/// of state -- throw the shield, Rush, place the shadow, raise a structure --
/// with no frames of its own and nothing to tune. The Blood mage's mechanic is
/// *health*, which is not a thing you press a button to change, so her `E` is
/// free to be an ability instead, and an ability needs a startup, a reach and a
/// cost like any other.
///
/// A function rather than a fixed slot index, because "the fourth slot" stopped
/// meaning "the `E` key" the moment a class had nineteen of them: the
/// Champion's fourth is the second link of its sword chain.
pub const fn on_e(class: Class) -> Option<u8> {
    match class {
        Class::BloodMage => Some(SLOTS as u8),
        // The Dual mage for the same reason, arrived at from the other
        // direction: her mechanic is a *meter*, and it is steered by which
        // button attacks rather than by a key. There is nothing for `E` to
        // toggle either, so it carries Sweep.
        Class::DualMage => Some(dual::SWEEP),
        // The Reaver is the odd one, and the only class where `E` carries a
        // move that is **not** the mechanic. Her mechanic is on right click,
        // because it is the half of her kit the crosshair aims; what is left
        // for the key is the swing, which does not care where it is thrown
        // from. See the note in [`NAMES`].
        Class::ShadowReaver => Some(crate::state::SLOT_COMMITTED),
        _ => None,
    }
}

/// Total move slots across the roster. The size of the Oven's move store.
pub const TOTAL_SLOTS: usize = {
    let mut n = 0;
    let mut i = 0;
    while i < crate::class::ALL_CLASSES.len() {
        n += slots(crate::class::ALL_CLASSES[i]);
        i += 1;
    }
    n
};

/// The most slots any one class has -- the Champion's nineteen.
///
/// Counted rather than written down, for the same reason [`TOTAL_SLOTS`] is: a
/// literal is a second statement of the same fact, and the two disagree the
/// first time somebody gives a class another move. It is the width of the
/// per-move storage a *fighter* carries, which today is the repeat lockout.
pub const MAX_SLOTS: usize = {
    let mut n = 0;
    let mut i = 0;
    while i < crate::class::ALL_CLASSES.len() {
        let s = slots(crate::class::ALL_CLASSES[i]);
        if s > n {
            n = s;
        }
        i += 1;
    }
    n
};

/// Where a class's slots begin in that store.
pub const fn base_slot(class: Class) -> usize {
    let mut n = 0;
    let mut i = 0;
    while i < crate::class::ALL_CLASSES.len() {
        if crate::class::ALL_CLASSES[i] as usize == class as usize {
            return n;
        }
        n += slots(crate::class::ALL_CLASSES[i]);
        i += 1;
    }
    n
}

/// The keys that throw a given slot's move.
///
/// Kept next to the move table so the Oven can print it beside the numbers: the
/// first question anyone asks about a knob is "which button is this", and
/// answering it in the same header removes a lookup from every tuning pass.
/// The bindings themselves are read by `crates/game` and documented in
/// `crates/manual`; this is the label, not the binding.
pub const fn binding(class: Class, slot: usize) -> &'static str {
    match class {
        // The grid in `champion`: the button is the weapon, the row is the
        // situation -- and the first three rows are one chain.
        Class::Champion => match slot {
            0 => "LMB",
            1 => "MMB",
            2 => "RMB",
            3 => "LMB, 2nd",
            4 => "MMB, 2nd",
            5 => "RMB, 2nd",
            6 => "LMB, 3rd",
            7 => "MMB, 3rd",
            8 => "RMB, 3rd",
            9 => "LMB air",
            10 => "MMB air",
            11 => "RMB air",
            12 => "LMB rush",
            13 => "MMB rush",
            14 => "RMB rush",
            15 => "Space+LMB",
            16 => "Space+MMB",
            17 => "Space+RMB",
            _ => "RMB rush, low",
        },
        // The Reaver's committed melee answers to right click as well, because
        // right click is otherwise dead on a class with no shield to raise.
        Class::ShadowReaver => match slot {
            0 => "LMB",
            1 => "E, Shift+LMB",
            2 => "Q",
            _ => "RMB",
        },
        // Both clicks are attacks, because the two autos are the mechanic: the
        // button is which force you throw and therefore which way you drift.
        // See [`dual`].
        Class::DualMage => match slot {
            0 => "LMB",
            1 => "MMB, light",
            2 => "Q",
            3 => "E",
            4 => "RMB",
            _ => "MMB, dark",
        },
        // Right click is otherwise dead weight on a class with no shield, the
        // same argument the Reaver makes -- Cataclysm takes it instead.
        Class::Elementalist => match slot {
            0 => "LMB",
            1 => "Shift+LMB",
            2 => "Q",
            3 => "RMB",
            // The air row. The button is the same; the situation is what
            // changes what it throws. See [`elementalist`].
            4 => "LMB air",
            5 => "RMB air",
            _ => "E air",
        },
        _ => match slot {
            0 => "LMB",
            1 => "Shift+LMB",
            2 => "Q",
            // The two classes with a fourth put an ability on the mechanic key:
            // the Blood mage because her mechanic is health and has nothing to
            // toggle, the Reaver because throwing her second body across the
            // arena is not an instant.
            _ => "E",
        },
    }
}

/// The volume a move puts in the world.
///
/// Code rather than a knob, and deliberately: the *span* of a swing is a
/// number somebody should be able to drag (it is `Move::arc`), but whether a
/// weapon sweeps or thrusts is what the move **is**. A slider that could turn
/// the spear into a hammer would not be a tuning knob, it would be a second,
/// worse way of writing the move list.
pub const fn shape(class: Class, kind: u8) -> Shape {
    use crate::aim::Hand;
    use champion as c;
    match class {
        // **A weapon owns a piece of space, and every one of its moves works
        // that piece.** The sword cuts corner to corner, the hammer travels up
        // or down the vertical plane the aim lies in, the spear is a line --
        // through all three links of the chain, in the air and out of a Rush.
        // That is what "each weapon has an identity" means when it is written as
        // code rather than as prose: a player who has learnt that the hammer
        // owns the ground under it has learnt something that is true of every
        // hammer move there is. `crates/sim/tests/feel.rs` pins the half of that
        // which is a design decision -- no two weapons ever put out the same
        // volume, so the shape on the screen always names the weapon.
        //
        // Three moves leave their weapon's usual plane, and each of them has the
        // same excuse: the situation took the plane away. Off the ground there is
        // no floor to cut across, so the sword rolls its arc into the vertical
        // and the spear sweeps its fan flat around the aim; and a Rush slash is
        // thrown at a line of people you are running through, which is a
        // horizontal cut by definition.
        Class::Champion => match kind {
            // **Corner to corner, and the two openers mirror each other.** The
            // sword owns the diagonal: down from the right shoulder, back down
            // from the left, and the finisher up the same line the first one
            // came down. A cut rolled off the vertical owns the width of the
            // front and the height of a body at once, which is why the sword is
            // the weapon you throw when you do not know where they will be.
            c::SWORD_GROUND => Shape::Swing(Plane::Diagonal(Hand::Right)),
            // And the second and third both hang off the other shoulder, which
            // is what makes the three of them one continuous form rather than
            // three swings: the first comes down to the left, so the second
            // starts from the left, and the third rises out of where the second
            // finished. Three strokes of a triangle, and the blade never has to
            // be carried back to a chamber.
            c::BACKCUT | c::UPCUT => Shape::Swing(Plane::Diagonal(Hand::Left)),
            // Across the front at chest height, which is what running through a
            // line of people looks like.
            c::RUSH_SLASH => Shape::Swing(Plane::Flat),
            // Up and down the vertical plane the aim lies in. The hammer owns
            // the line under it: overhead to the floor, torn back out of it,
            // and driven through it.
            c::HAMMER_GROUND | c::UPROOT | c::EARTHBREAKER | c::RUSH_SWEEP => {
                Shape::Swing(Plane::Upright)
            }
            // Straight out along the aim. The spear owns the distance -- and its
            // finisher spends that length the other way round, swept flat all
            // the way round the body, which is the one thing a three-metre pole
            // can do that nothing else in the game can. It is still a line; it
            // is the circle the line draws that is new.
            c::SPEAR_GROUND | c::SKEWER | c::RUSH_STAB => Shape::Thrust,
            c::WHIRL => Shape::Swing(Plane::Flat),
            // The air: a sword cut you bring down on somebody, a hammer you
            // drop on them, and a fan the spear sweeps around wherever you are
            // pointing.
            c::AIR_SWORD | c::AIR_HAMMER => Shape::Swing(Plane::Upright),
            c::AIR_SPEAR => Shape::Swing(Plane::Flat),
            // The takeoffs are all vertical, because all three of them are the
            // weapon going the way the body is about to: the sword and the
            // hammer rise through the arc and the spear drives the other end of
            // itself into the floor.
            c::RISING_CUT | c::UPPERCUT | c::POLE_DRIVE => Shape::Swing(Plane::Upright),
            // Movement, not an attack. A vault that also hit people would be
            // strictly better than the stab it shares a button with.
            _ => Shape::None,
        },
        // Her two air shots put a thing in the world and let it do the
        // hitting, so the body itself has no volume at all -- the same shape
        // the Champion's vault is, arrived at from the other direction. The
        // Blood mage's blade says the same thing with a radius of zero; these
        // cannot, because a travelling shot needs a thickness and `radius` is
        // where it is written. See `Move::strikes` for the two sentences.
        //
        // Everything else she has is still the original disc: three of the
        // four grounded moves land on the floor where they were aimed, the
        // fourth is a beam drawn from the line it flew, and Landfall is a disc
        // on the floor at her own feet.
        Class::Elementalist => match kind {
            elementalist::AIR_BOLT | elementalist::GALE => Shape::None,
            _ => Shape::Cylinder,
        },
        // The Dual mage's two autos are punches with a wing behind them, and
        // Sweep is a cut across the whole front. Lance and Judgement are still
        // discs: one is a skillshot, whose volume is the line it flew, and the
        // other lands on the floor where it was aimed.
        Class::DualMage => match kind {
            dual::DARK_AUTO | dual::LIGHT_AUTO => Shape::Wing,
            dual::SWEEP => Shape::Swing(Plane::Flat),
            // The dark Lance has no volume of its own: the line it throws is
            // the effect, and the effect is what catches somebody and holds
            // on. See `effects::EffectKind::Tether`.
            dual::DARK_LANCE => Shape::None,
            _ => Shape::Cylinder,
        },
        // Every other class is still the original disc at arm's length.
        _ => Shape::Cylinder,
    }
}

/// Which arm a move comes out of.
///
/// Code rather than a knob, and for the same reason [`shape`] is: which hand
/// throws a punch is what the move **is**. A slider that moved a hitbox from one
/// arm to the other would be a way of making the animation and the hit test
/// disagree, which is the one thing they may never do.
///
/// [`crate::aim::Hand::Centre`] for all but two moves in the game, which is the
/// body's own line and is where every volume sat before the Dual mage needed to
/// tell her two arms apart.
pub const fn hand(class: Class, kind: u8) -> crate::aim::Hand {
    use crate::aim::Hand;
    match class {
        // Left is dark, right is light. The class in one line: see
        // `docs/design/dual-mage.md`.
        //
        // **And the two Lances follow the same rule**, which is why they are on
        // the list rather than down at `Hand::Centre` with everything else that
        // is not an auto. Middle click throws whichever form the force in her
        // arms decides, so the arm it leaves from is a second reading of the
        // same fact: the burst comes off the light hand and the tether off the
        // dark one. A thrust out of the sternum would say nothing at all, and
        // on this class saying nothing is the bug.
        Class::DualMage => match kind {
            dual::DARK_AUTO | dual::DARK_LANCE => Hand::Left,
            dual::LIGHT_AUTO | dual::LIGHT_LANCE => Hand::Right,
            _ => Hand::Centre,
        },
        // The spear's opener is the one attack in the class thrown with one
        // arm: a jab off the leading hand, with the butt of the shaft still
        // tucked at the hip. Everything else the Champion throws is on the haft
        // with both hands, so the grip is the body's own centre line -- and the
        // jab coming out of a shoulder instead is most of why it reads as a
        // poke rather than as a short version of the lunge it replaced.
        Class::Champion if kind == champion::SPEAR_GROUND => Hand::Right,
        _ => Hand::Centre,
    }
}

/// Does this ability stay **out in the world** after the cast, so that its
/// repeat lockout has to wait for it?
///
/// Two abilities do, and both are the Reaver's: the shadow stands where it was
/// sent until it is called back, and the lotus hangs its blades until a recall
/// drags them home. An ability is not *used* until it is spent, so their
/// lockouts are parked at full for as long as any of it is still out there --
/// otherwise leaving the shadow parked in a corner would quietly serve the
/// lockout for the next send while it waited.
///
/// **Declared here, and what it declares is the question rather than the
/// answer.** Whether the thing is still out is a fact about the world and lives
/// in `state::Player::abilities_out`, which is the only place that knows how to
/// look -- the shadow is the class mechanic and the blades are an entry in the
/// effect table, and nothing but that function knows both. This says which
/// slots it is worth asking about, which is what lets the frame table say
/// "waits until it is home" where it would otherwise print a frame count that
/// is only true for an ability that finishes when its recovery does.
///
/// An ability that lingers but has no answer in `abilities_out` simply gets the
/// ordinary rule, which is the safe direction to be wrong in: its lockout
/// starts at the cast rather than never starting at all.
pub const fn lingers(class: Class, kind: u8) -> bool {
    matches!(class, Class::ShadowReaver)
        && matches!(
            kind,
            crate::state::SLOT_MECHANIC | crate::state::SLOT_SPECIAL
        )
}

/// Does this slot's own button, pressed again, **reactivate** what the first
/// press put out there -- rather than throwing a second copy of it?
///
/// **Declared, not inferred**, the same way [`shape`] and [`hand`] are, and for
/// a sharper reason than either: the repeat lockout reads this to decide
/// whether a press is a new use of the ability or the rest of the one already
/// paid for, and getting that backwards either eats the recall or makes the
/// ability free to spam. Inferring it from "does this class leave something in
/// the world" answers yes for the fire pillar and the black spike, neither of
/// which can be pressed again at all.
///
/// One move in the game says yes. **Send shadow** is one button with two
/// meanings decided by where the second body is -- out, or home -- so the press
/// that brings it back is the second half of the send rather than another send.
///
/// The **Guillotine lotus** is the near miss that shows why this is a question
/// about the *button* and not about the ability. It has a second activation --
/// recalling the shadow drags the hanging blades home, which is the combination
/// the whole kit is built around -- but that activation is on *right click*,
/// not on the lotus's own key. Pressing `Q` again would be a fresh lotus while
/// the first one is still hanging, so its answer is no.
pub const fn reactivates(class: Class, kind: u8) -> bool {
    matches!(class, Class::ShadowReaver) && kind == crate::state::SLOT_MECHANIC
}

/// Does this class use this slot at all?
///
/// Storage is **packed** to each class's own count, so this is now simply
/// "is the slot in range" -- there is no hole in the table to step over, and a
/// class that does not have a slot does not have knobs for one either. It
/// stays as a named predicate because the callers are asking a question about
/// the design rather than about an array bound: a move with all its numbers set
/// to zero is plus on block and kills in no hits, and would fail every property
/// in `feel.rs` for reasons that have nothing to do with the game.
pub fn bound(class: Class, slot: usize) -> bool {
    slot < slots(class)
}

/// Build a move from the live tuning store.
///
/// By value rather than by reference: the numbers can change between frames, so
/// there is no `'static` to borrow from any more. A `Move` is small and copied
/// a handful of times a frame.
pub fn get(class: Class, kind: u8) -> Move {
    use crate::oven::{self, MoveField as F};
    let slot = (kind as usize).min(slots(class) - 1);
    let raw = |f: F| oven::move_field(class, slot, f);
    Move {
        name: NAMES[class as usize][slot],
        startup: raw(F::Startup) as u16,
        active: raw(F::Active) as u16,
        recovery: raw(F::Recovery) as u16,
        damage: raw(F::Damage),
        reach: Fx::from_raw(raw(F::Reach)),
        radius: Fx::from_raw(raw(F::Radius)),
        hitstun: raw(F::Hitstun) as u16,
        blockstun: raw(F::Blockstun) as u16,
        knockback: Fx::from_raw(raw(F::Knockback)),
        unblockable: raw(F::Unblockable) != 0,
        hits_crouching: raw(F::HitsCrouching) != 0,
        needs_mechanic: raw(F::NeedsMechanic) != 0,
        mobility: raw(F::Mobility) as u8,
        air_stall: raw(F::AirStall) as u16,
        launch: Fx::from_raw(raw(F::Launch)),
        self_lift: Fx::from_raw(raw(F::SelfLift)),
        grabs: raw(F::Grabs) as u16,
        effect: raw(F::Effect) as u8,
        cost: raw(F::Cost),
        leech: raw(F::Leech) as u8,
        aim_code: raw(F::Aim) as u8,
        arc: Fx::from_raw(raw(F::Arc)),
        rehit: raw(F::Rehit) as u16,
        channel: raw(F::Channel) as u16,
        channel_from: Fx::from_raw(raw(F::ChannelFrom)),
        repeat_mul: raw(F::RepeatMul).clamp(0, 255) as u8,
        reactivate: raw(F::Reactivate).max(0) as u16,
        step: Fx::from_raw(raw(F::Step)),
        shape: shape(class, slot as u8),
        hand: hand(class, slot as u8),
    }
}

/// Every move a class actually has, live.
///
/// A `Vec` rather than an array because the count is not the same for
/// everybody: three for most of the roster, four for the Blood mage, nineteen
/// for the Champion. The callers are the frame table and the feel tests, neither of
/// which runs inside a frame, so the allocation buys readability for nothing.
pub fn table(class: Class) -> Vec<Move> {
    (0..slots(class)).map(|i| get(class, i as u8)).collect()
}

/// Frame data for a move, for debug overlays and documents.
pub fn frames(class: Class, kind: u8) -> (u16, u16, u16) {
    let m = get(class, kind);
    (m.startup, m.active, m.recovery)
}

// ---------------------------------------------------------------------------
// Where the weapon is this frame
// ---------------------------------------------------------------------------

/// The point a weapon is swung from, given where the fighter stands.
///
/// A swing in the vertical plane starts from the chest, because that is where
/// an overhead begins and where a rising cut is aimed from. A cut **across** the
/// body starts lower -- see `tuning::sweep_height`. A one-armed move starts a
/// shoulder's width to that side of both, which is `crate::aim::hand_origin`'s
/// business rather than this function's: where along the *body* a weapon hinges
/// is a question about the weapon, and which *side* it hinges on is a question
/// about the fighter.
pub fn swing_hub(pos: V3, facing: V3, plane: Plane, hand: crate::aim::Hand) -> V3 {
    let chest = crate::aim::hand_origin(pos, facing, hand);
    match plane {
        // Both hang off the hands where they are. A diagonal cut is an upright
        // one rolled over; the grip has not moved.
        Plane::Upright | Plane::Diagonal(_) => chest,
        Plane::Flat => V3::new(
            chest.x,
            pos.y
                .add(chest.y.sub(pos.y).mul(crate::tuning::sweep_height())),
            chest.z,
        ),
    }
}

/// The line a swing is measured from, before its own arc is applied.
pub fn swing_base(facing: V3, aim_dir: V3, plane: Plane, grounded: bool) -> V3 {
    match plane {
        // Across the body: level with the shoulders, along the facing,
        // whatever the camera is doing -- a cut does not go where you look, it
        // goes where your shoulders are. Dipped, because it is a low cut.
        Plane::Flat if grounded => turned(facing, crate::tuning::sweep_dip().neg(), Plane::Upright),
        // In the air the fan is thrown *around* the aim, so the climb comes
        // with it and a spear swept at the floor stays at the floor.
        Plane::Flat => aim_dir,
        // Down the body, in the plane the aim already lies in -- and across the
        // diagonal of it, which is the same plane rolled.
        Plane::Upright | Plane::Diagonal(_) => aim_dir,
    }
}

/// The ring a wing carves, placed in the world.
///
/// A wing is a section of a torus lying flat (see [`Shape::Wing`]), and this is
/// the whole torus plus the two bearings the section lives between. Everything
/// in it is a knob, because where this shape sits relative to the body that
/// threw it *is* the move -- see `tuning::wing_inner` and the three beside it.
///
/// ```text
///                       .-  -  -  -.
///                   . '             ' .        outer arc at `Move::reach`
///     starts -->  ;                     :      inner arc `wing_inner` of it
///     behind her   \                   /       and nothing in between
///        o          ' .     (x)   . '
///        |              ' - , - '     <-- finishes in front of that fist
///     the mage,                           `wing_finish` off her centre line
///     punching                        (x) the ring's middle: pushed
///     left-handed                         `wing_offside` toward the other
///                                         arm and `wing_ahead` forward
/// ```
///
/// The middle is pushed **off** her rather than sitting on her, which is what
/// makes the blade pass by rather than wrap round: a ring centred on a fighter
/// is the same distance from them at every bearing, and a piece of one reads as
/// a halo however short it is.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Wing {
    /// The middle of the ring, at the height the hand punches through.
    pub at: V3,
    pub inner: Fx,
    pub outer: Fx,
    /// The bearing the leading edge stops on, in front of the punching hand.
    pub finish: Fx,
    /// How far back from `finish` the section starts, **signed by the arm**:
    /// positive is the way [`crate::aim::Hand::Left`] sweeps. One tuned `arc`
    /// serves both autos because the sign lives here and nowhere else.
    pub span: Fx,
}

/// Where a wing's ring is, given where the fighter stands and what they threw.
///
/// The plane of it is the floor's while her feet are on it, which is the one
/// place this move ignores the camera's pitch. Off the ground there is no shared
/// floor to lie parallel to, so it rides the aim -- and for a flat ring that
/// means only its height moves, because a ring lying in the aim's plane is
/// still a ring. The same split `swing_base` already makes for a cut thrown in
/// the air.
pub fn wing(pos: V3, facing: V3, aim_dir: V3, grounded: bool, m: &Move) -> Wing {
    use crate::tuning as t;
    let middle = crate::aim::origin(pos)
        .add(facing.scale(m.reach.mul(t::wing_ahead())))
        .sub(crate::aim::across(facing, m.hand).scale(m.reach.mul(t::wing_offside())));
    let at = if grounded {
        middle
    } else {
        V3::new(middle.x, middle.y.add(aim_dir.y.mul(m.reach)), middle.z)
    };
    let ahead = crate::math::atan2_turns(facing.z, facing.x);
    let outward = Fx::from_int(m.hand.outward());
    Wing {
        at,
        inner: m.reach.mul(t::wing_inner()),
        outer: m.reach,
        finish: ahead.add(t::wing_finish().mul(outward)),
        span: m.arc.mul(outward),
    }
}

impl Wing {
    /// The section that is actually out, `through` of the way through the
    /// opening.
    ///
    /// **It opens rather than sweeps.** The trailing edge stays where the punch
    /// threw it and the leading edge comes round toward the front, so the shape
    /// is a wing spreading rather than a blade travelling -- the beings inside
    /// her extending the movement past where an arm could take it.
    ///
    /// It stops `tuning::wing_tip` of the span short of the finish. That last
    /// piece belongs to the tip, which is a bubble rather than a section: see
    /// [`Wing::tip`].
    pub fn opened(&self, through: Fx) -> crate::math::Sector {
        let short = self.span.mul(crate::tuning::wing_tip());
        let leading = short.add(self.span.sub(short).mul(Fx::ONE.sub(through)));
        crate::math::Sector {
            at: self.at,
            inner: self.inner,
            outer: self.outer,
            from: self.finish.add(self.span),
            to: self.finish.add(leading),
        }
    }

    /// The foremost point of the ring: the outer arc at the bearing the wing
    /// finishes on, which is where the tip arrives on the last active frame.
    ///
    /// Read off a section with no width rather than worked out again here. A
    /// second copy of the same arithmetic is how a tip ends up somewhere the
    /// wing it belongs to never went.
    pub fn tip(&self) -> V3 {
        crate::math::Sector {
            at: self.at,
            inner: self.inner,
            outer: self.outer,
            from: self.finish,
            to: self.finish,
        }
        .point(Fx::ONE, Fx::ONE)
    }
}

/// Turn a direction by `by` turns, in one of the two planes a weapon swings
/// in.
///
/// Exact integer trigonometry either way: `Flat` is a rotation of the
/// horizontal components about the vertical axis with the climb left alone,
/// and `Upright` rotates the climb against the horizontal length in the plane
/// the direction already lies in. Neither needs an inverse trig function, which
/// is the only reason this is cheap enough to do inside the hit test.
pub fn turned(base: V3, by: Fx, plane: Plane) -> V3 {
    let (c, s) = (crate::fixed::cos_turns(by), crate::fixed::sin_turns(by));
    match plane {
        Plane::Flat => V3::new(
            base.x.mul(c).sub(base.z.mul(s)),
            base.y,
            base.x.mul(s).add(base.z.mul(c)),
        ),
        Plane::Upright => {
            let flat = V3::new(base.x, Fx::ZERO, base.z);
            let run = flat.flat_len();
            let along = flat.normalized();
            // A rotation in the (run, climb) plane, with the sign chosen so
            // that a positive angle lifts the head of the weapon. Swings are
            // written with a positive `arc` meaning "comes down", and the
            // sweep below starts at `+arc/2`, which is therefore up.
            let run2 = run.mul(c).sub(base.y.mul(s));
            let climb = base.y.mul(c).add(run.mul(s));
            V3::new(along.x.mul(run2), climb, along.z.mul(run2))
        }
        // The same rotation as `Upright`, about an axis rolled off the
        // horizontal -- so the head travels corner to corner instead of
        // straight down. Written as "which way is up for this swing" rather
        // than as an axis, because that is the one line of it worth reading:
        // the aim is the middle of the arc either way, and all the plane
        // decides is where the far end of it starts.
        Plane::Diagonal(hand) => {
            let aim = base.normalized();
            let (side, up) = crate::math::frame_about(aim);
            // `frame_about`'s sideways axis is the one `aim::across` hands out
            // for the **left** hand, so the roll takes the sign of the hand
            // directly and a `Diagonal(Right)` cut starts over the right
            // shoulder. One tuned tilt, two mirrored cuts.
            let roll = crate::tuning::cut_roll().mul(Fx::from_int(hand.outward()));
            let high = up
                .scale(crate::fixed::cos_turns(roll))
                .add(side.scale(crate::fixed::sin_turns(roll)));
            aim.scale(c).add(high.scale(s))
        }
    }
}
