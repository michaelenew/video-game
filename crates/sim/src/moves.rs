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
    /// Percent of walking speed you keep while the move runs.
    ///
    /// Zero roots you, which is what commitment means and is correct for the
    /// heavy moves. It is wrong for a fast poke: the poke is the neutral tool,
    /// thrown constantly, and stopping dead every time makes neutral sticky and
    /// reads as the game snatching the controls away. Slowing you keeps the
    /// cost -- you cannot close or escape at full speed while swinging --
    /// without the lurch.
    pub mobility: u8,
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

/// Three exemplar moves per class, in slot order: poke, committed, special.
const NAMES: [[&str; SLOTS]; 6] = [
    // Bulwark -- committed, not slow. Wins by denying space.
    //   Bash: fast poke, slightly minus on block so it is not a free mash.
    //   Slam: the overhead. Heavily punishable if read, heavily rewarding if not.
    //   Grapple: beats guard outright, loses badly to dodge.
    ["Bash", "Slam", "Grapple"],
    // Bellator -- range bands and flow. Form multiplies everything.
    ["Sweep", "Drive", "Uppercut"],
    // Shadow Reaver -- two bodies. Options are a function of the line between them.
    //   Guillotine: blades erupt from the shadow, so it needs one placed.
    ["Slash", "Executioner", "Guillotine"],
    // Elementalist -- terrain author. Ranged, and creates its own targets.
    //   Fire pillar: detonates a structure for a wider blast, so it wants one out.
    ["Bolt", "Fissure", "Fire pillar"],
    // Blood mage -- sustain through aggression. Everything costs health.
    //   Reaper's debt: committed and directional, you cannot turn while it channels.
    ["Rend", "Black spike", "Reaper's debt"],
    // Dual mage -- melee mage riding between two forces.
    //   Judgement: a finisher, only past the deep threshold on its own side.
    ["Step strike", "Lance", "Judgement"],
];

pub const SLOTS: usize = 3;

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
    }
}

/// All three of a class's moves, live.
pub fn table(class: Class) -> [Move; SLOTS] {
    [get(class, 0), get(class, 1), get(class, 2)]
}

/// Frame data for a move, for debug overlays and documents.
pub fn frames(class: Class, kind: u8) -> (u16, u16, u16) {
    let m = get(class, kind);
    (m.startup, m.active, m.recovery)
}
