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
    /// Upward speed given to whoever this hits. The Champion's uppercut takes
    /// people into the air with it; most moves leave them on the ground.
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
    /// Zero roots you, which is what commitment means and is correct for the
    /// heavy moves. It is wrong for a fast poke: the poke is the neutral tool,
    /// thrown constantly, and stopping dead every time makes neutral sticky and
    /// reads as the game snatching the controls away. Slowing you keeps the
    /// cost -- you cannot close or escape at full speed while swinging --
    /// without the lurch.
    pub mobility: u8,
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

    /// Whether this move strikes on its own, or only places something.
    ///
    /// A radius of zero is not a tiny hitbox, it is *no* hitbox: the move is a
    /// gesture that puts something into the world and the thing it put there
    /// does all of the hitting. The Blood mage's auto and her Grasp are both
    /// that shape -- the blade and the arms carry the damage, and the caster's
    /// own body never touches anybody.
    pub const fn strikes(&self) -> bool {
        self.radius.raw() > 0
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

/// The exemplar moves of each class, in slot order: poke, committed, special,
/// and the mechanic.
///
/// **The fourth slot is the `E` key**, and it is empty for most of the roster.
/// `E` is the class mechanic, and for five of the six classes the mechanic is
/// an instant change of state -- throw the shield, cycle the form, place the
/// shadow, raise a structure -- with no frames of its own and nothing to tune.
/// The Blood mage's mechanic is *health*, which is not a thing you press a
/// button to change, so her `E` is free to be an ability instead, and an
/// ability needs a startup, a reach and a cost like any other. An empty name
/// means the class does not bind the slot; see [`bound`].
const NAMES: [[&str; SLOTS]; 6] = [
    // Bulwark -- committed, not slow. Wins by denying space.
    //   Bash: fast poke, slightly minus on block so it is not a free mash.
    //   Slam: the overhead. Heavily punishable if read, heavily rewarding if not.
    //   Grapple: beats guard outright, loses badly to dodge.
    ["Bash", "Slam", "Grapple", ""],
    // Champion -- range bands and flow. Form multiplies everything.
    ["Sweep", "Drive", "Uppercut", ""],
    // Shadow Reaver -- two bodies. Options are a function of the line between them.
    //   Guillotine: blades erupt from the shadow, so it needs one placed.
    ["Slash", "Executioner", "Guillotine", ""],
    // Elementalist -- terrain author. Ranged, and creates its own targets.
    //   Bolt: the game's one *skillshot* -- an instant line from her hand to
    //   whatever the crosshair is on, so its `reach` is the max-range sphere
    //   and its `radius` is the line's thickness. Resolved where it is fired
    //   rather than by the hitbox loop, and its hitstun and knockback are zero
    //   on purpose: it takes the charge off whoever it catches and gives them
    //   their frames straight back. See `crate::bolt`.
    ["Bolt", "Fissure", "Fire pillar", ""],
    // Blood mage -- sustain through aggression. Everything costs health, and
    // every one of these has a cost in the table to prove it.
    //   Bloodletter: the auto. Out to a fixed distance and back, cutting on
    //     both passes, and the blood it takes comes home with it.
    //   Rend: the committed poke it was before the auto took its slot.
    //   Grasp: four arms out in a cone that arc back inward to meet. Caught by
    //     all four and you are rooted.
    //   Black spike: on `E`, because the class has no other use for the key.
    ["Bloodletter", "Rend", "Grasp", "Black spike"],
    // Dual mage -- melee mage riding between two forces.
    //   Judgement: a finisher, only past the deep threshold on its own side.
    ["Step strike", "Lance", "Judgement", ""],
];

pub const SLOTS: usize = 4;

/// The keys that throw a given slot's move.
///
/// Kept next to the move table so the Oven can print it beside the numbers: the
/// first question anyone asks about a knob is "which button is this", and
/// answering it in the same header removes a lookup from every tuning pass.
/// The bindings themselves are read by `crates/game` and documented in
/// `crates/manual`; this is the label, not the binding.
pub const fn binding(slot: usize) -> &'static str {
    match slot {
        0 => "LMB",
        1 => "Shift+LMB",
        2 => "Q",
        _ => "E",
    }
}

/// Does this class use this slot at all?
///
/// Only ever false for the mechanic slot, and only because `E` means something
/// different on every class -- see [`NAMES`]. Everything that walks the move
/// table filters on this, so an unbound slot is a hole in the table rather than
/// a move with all its numbers set to zero, which is a move that is plus on
/// block and kills in no hits and would fail every property in `feel.rs` for
/// reasons that have nothing to do with the game.
pub fn bound(class: Class, slot: usize) -> bool {
    slot < SLOTS && !NAMES[class as usize][slot].is_empty()
}

/// Build a move from the live tuning store.
///
/// By value rather than by reference: the numbers can change between frames, so
/// there is no `'static` to borrow from any more. A `Move` is small and copied
/// a handful of times a frame.
pub fn get(class: Class, kind: u8) -> Move {
    use crate::oven::{self, MoveField as F};
    let slot = (kind as usize).min(SLOTS - 1);
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
    }
}

/// Every move a class actually has, live.
///
/// A `Vec` rather than an array because the count is no longer the same for
/// everybody: the mechanic slot is bound on one class and empty on five. The
/// callers are the frame table and the feel tests, neither of which runs inside
/// a frame, so the allocation buys readability for nothing.
pub fn table(class: Class) -> Vec<Move> {
    (0..SLOTS)
        .filter(|slot| bound(class, *slot))
        .map(|slot| get(class, slot as u8))
        .collect()
}

/// Frame data for a move, for debug overlays and documents.
pub fn frames(class: Class, kind: u8) -> (u16, u16, u16) {
    let m = get(class, kind);
    (m.startup, m.active, m.recovery)
}
