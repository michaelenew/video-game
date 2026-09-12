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
    /// Percent of walking speed you keep while the move runs.
    ///
    /// Zero roots you, which is what commitment means and is correct for the
    /// heavy moves. It is wrong for a fast poke: the poke is the neutral tool,
    /// thrown constantly, and stopping dead every time makes neutral sticky and
    /// reads as the game snatching the controls away. Slowing you keeps the
    /// cost -- you cannot close or escape at full speed while swinging --
    /// without the lurch.
    pub mobility: u8,
    /// How far a swing travels, in turns, and **which way**.
    ///
    /// Signed: a positive arc runs from the `+` end of its span to the `-` end
    /// -- right to left across the body, or top to bottom down it -- and a
    /// negative one reverses, which is what makes an uppercut a rising cut
    /// rather than a falling one from the same two numbers. Zero is a move
    /// whose shape does not sweep.
    pub arc: Fx,
    /// Frames between one connection and the next for a move that keeps
    /// hitting. Zero is the normal rule: a swing lands once.
    ///
    /// It exists for exactly one move -- the Champion's Rush slash, which is
    /// meant to be run *through* a crowd -- and it is the one place where the
    /// on-hit arithmetic below stops meaning anything, because the move is
    /// still in its active frames when the next cut lands.
    pub rehit: u16,
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
    /// The volume this move puts in the world. See [`Shape`].
    pub shape: Shape,
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
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Plane {
    /// Across the body, turning about the vertical. A cut that catches
    /// everything in front of you and nothing above or below it.
    Flat,
    /// Down the body, turning in the vertical plane that contains the aim. A
    /// slam, a rising cut, and everything the Champion does in the air.
    Upright,
}

impl Move {
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

    /// Whether the move pins you in place for its duration.
    pub const fn roots(&self) -> bool {
        self.mobility == 0
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
/// a cost like any other. The Champion has ten, because its three mouse buttons
/// are three weapons and each of them behaves differently on foot, in the air
/// and mid-Rush: see [`champion`].
///
/// Storage is packed to these counts, so a class that does not have a slot does
/// not have knobs for one either -- see [`slots`] and [`bound`].
const NAMES: [&[&str]; 6] = [
    // Bulwark -- committed, not slow. Wins by denying space.
    //   Bash: fast poke, slightly minus on block so it is not a free mash.
    //   Slam: the overhead. Heavily punishable if read, heavily rewarding if not.
    //   Grapple: beats guard outright, loses badly to dodge.
    &["Bash", "Slam", "Grapple"],
    // Champion -- three weapons on three buttons, and a dash that changes what
    // all three of them do. Ten moves: see `champion` for the grid they form.
    &[
        "Sword",
        "Hammer",
        "Spear",
        "Air sword",
        "Air hammer",
        "Air spear",
        "Rush slash",
        "Uppercut",
        "Rush stab",
        "Pole vault",
    ],
    // Shadow Reaver -- two bodies. Options are a function of the line between them.
    //   Guillotine: blades erupt from the shadow, so it needs one placed.
    &["Slash", "Executioner", "Guillotine"],
    // Elementalist -- terrain author. Ranged, and creates its own targets.
    //   Bolt: the odd one out. It is a *beam* -- an instant ray along the
    //   crosshair -- so its `reach` is the line's length and its `radius` is the
    //   line's thickness, and it is resolved where it is fired rather than by
    //   the hitbox loop. Its hitstun and knockback are zero on purpose: it takes
    //   the charge off whoever it catches and gives them their frames straight
    //   back. See `crate::bolt`.
    &["Bolt", "Fissure", "Fire pillar"],
    // Blood mage -- sustain through aggression. Everything costs health, and
    // every one of these has a cost in the table to prove it.
    //   Bloodletter: the auto. Out to a fixed distance and back, cutting on
    //     both passes, and the blood it takes comes home with it.
    //   Rend: the committed poke it was before the auto took its slot.
    //   Grasp: four arms out in a cone that arc back inward to meet. Caught by
    //     all four and you are rooted.
    //   Black spike: on `E`, because the class has no other use for the key.
    &["Bloodletter", "Rend", "Grasp", "Black spike"],
    // Dual mage -- melee mage riding between two forces.
    //   Judgement: a finisher, only past the deep threshold on its own side.
    &["Step strike", "Lance", "Judgement"],
];

/// The slots every class has: poke, committed, special. Two classes have more.
pub const SLOTS: usize = 3;

// ---------------------------------------------------------------------------
// The Champion's ten
// ---------------------------------------------------------------------------

/// The Champion's move list, as a **grid**: three stances by three weapons,
/// plus the one move that is neither.
///
/// This is the whole of the class's input scheme and it is worth reading as a
/// table rather than as a list of ten moves:
///
/// ```text
///              left click      middle click    right click
///   on foot    Sword           Hammer          Spear
///   in the air Air sword       Air hammer      Air spear
///   rushing    Rush slash      Uppercut        Rush stab / Pole vault
/// ```
///
/// The button is the **weapon** and never changes meaning; the row is where
/// your feet are. That is the entire thing a new player has to learn, and it
/// is why the Champion can carry ten moves on three buttons without a single
/// modifier: you never choose a move, you choose a weapon, and the situation
/// chooses the move.
///
/// The tenth is the Pole vault, which shares right click with the Rush stab
/// and is separated by where you are pointing: a spear planted in the ground
/// vaults, a spear levelled at someone stabs.
pub mod champion {
    /// Weapons, in button order. The column of the grid.
    pub const SWORD: u8 = 0;
    pub const HAMMER: u8 = 1;
    pub const SPEAR: u8 = 2;

    /// Stances. The row of the grid, as a base to add a weapon to.
    pub const ON_FOOT: u8 = 0;
    pub const IN_THE_AIR: u8 = 3;
    pub const RUSHING: u8 = 6;

    pub const SWORD_GROUND: u8 = ON_FOOT + SWORD;
    pub const HAMMER_GROUND: u8 = ON_FOOT + HAMMER;
    pub const SPEAR_GROUND: u8 = ON_FOOT + SPEAR;
    pub const AIR_SWORD: u8 = IN_THE_AIR + SWORD;
    pub const AIR_HAMMER: u8 = IN_THE_AIR + HAMMER;
    pub const AIR_SPEAR: u8 = IN_THE_AIR + SPEAR;
    pub const RUSH_SLASH: u8 = RUSHING + SWORD;
    pub const UPPERCUT: u8 = RUSHING + HAMMER;
    pub const RUSH_STAB: u8 = RUSHING + SPEAR;
    /// The odd one out: right click during a Rush, aimed at the floor.
    pub const POLE_VAULT: u8 = 9;

    pub const COUNT: usize = 10;

    /// Which weapon a move is thrown with. The column of the grid, except for
    /// the vault, which is planted with the spear like everything else on
    /// right click.
    pub const fn weapon(kind: u8) -> u8 {
        if kind == POLE_VAULT { SPEAR } else { kind % 3 }
    }
}

/// How many moves a class has.
///
/// Per class rather than a single constant because the Champion legitimately
/// grew: its three mouse buttons are three weapons and each weapon behaves
/// differently on foot, in the air and mid-Rush, which is nine moves plus the
/// vault. Every other class still has the original three, and the storage in
/// the Oven is packed to these counts so the five that did not grow cost
/// nothing.
pub const fn slots(class: Class) -> usize {
    match class {
        Class::Champion => champion::COUNT,
        // The fourth is Black spike, on `E`. See `on_e`.
        Class::BloodMage => SLOTS + 1,
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
/// meaning "the `E` key" the moment a class had ten of them: the Champion's
/// fourth is its aerial sword.
pub const fn on_e(class: Class) -> Option<u8> {
    match class {
        Class::BloodMage => Some(SLOTS as u8),
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
        // The grid in `champion`: the button is the weapon, the row is where
        // your feet are.
        Class::Champion => match slot {
            0 => "LMB",
            1 => "MMB",
            2 => "RMB",
            3 => "LMB air",
            4 => "MMB air",
            5 => "RMB air",
            6 => "LMB rush",
            7 => "MMB rush",
            8 => "RMB rush",
            _ => "RMB rush, low",
        },
        _ => match slot {
            0 => "LMB",
            1 => "Shift+LMB",
            2 => "Q",
            // Only the Blood mage has a fourth, because only she has no use
            // for `E` as a mechanic key -- her mechanic is health.
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
    use champion as c;
    match class {
        Class::Champion => match kind {
            // Across the front, at hip height. The sword owns the width.
            c::SWORD_GROUND => Shape::Swing(Plane::Flat),
            // Overhead to the floor. The hammer owns the line under it.
            c::HAMMER_GROUND => Shape::Swing(Plane::Upright),
            // Straight out along the aim. The spear owns the distance.
            c::SPEAR_GROUND => Shape::Thrust,
            // The same three, rolled into the vertical: a sword cut you bring
            // down on somebody, a hammer you drop on them, and a fan the spear
            // sweeps around wherever you are pointing.
            c::AIR_SWORD | c::AIR_HAMMER => Shape::Swing(Plane::Upright),
            c::AIR_SPEAR => Shape::Swing(Plane::Flat),
            c::RUSH_SLASH => Shape::Swing(Plane::Flat),
            c::UPPERCUT => Shape::Swing(Plane::Upright),
            c::RUSH_STAB => Shape::Thrust,
            // Movement, not an attack. A vault that also hit people would be
            // strictly better than the stab it shares a button with.
            _ => Shape::None,
        },
        // Every other class is still the original disc at arm's length.
        _ => Shape::Cylinder,
    }
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
        arc: Fx::from_raw(raw(F::Arc)),
        rehit: raw(F::Rehit) as u16,
        shape: shape(class, slot as u8),
    }
}

/// Every move a class actually has, live.
///
/// A `Vec` rather than an array because the count is not the same for
/// everybody: three for most of the roster, four for the Blood mage, ten for
/// the Champion. The callers are the frame table and the feel tests, neither of
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
/// body starts lower -- see `tuning::sweep_height`.
pub fn swing_hub(pos: V3, plane: Plane) -> V3 {
    let chest = crate::aim::origin(pos);
    match plane {
        Plane::Upright => chest,
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
        // Down the body, in the plane the aim already lies in.
        Plane::Upright => aim_dir,
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
    }
}
