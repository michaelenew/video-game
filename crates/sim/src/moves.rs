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
use crate::tuning::POKE_MOBILITY;

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

/// Shorthand so the tables below stay readable.
const fn mv(
    name: &'static str,
    frames: (u16, u16, u16),
    damage: i32,
    reach: (i32, i32),
    radius: (i32, i32),
    stun: (u16, u16),
    knockback: i32,
) -> Move {
    Move {
        name,
        startup: frames.0,
        active: frames.1,
        recovery: frames.2,
        damage,
        reach: Fx::ratio(reach.0, reach.1),
        radius: Fx::ratio(radius.0, radius.1),
        hitstun: stun.0,
        blockstun: stun.1,
        knockback: Fx::ratio(knockback, 1),
        unblockable: false,
        hits_crouching: true,
        needs_mechanic: false,
        mobility: 0,
    }
}

/// Keep this fraction of walking speed through the move.
const fn mobile(mut m: Move, percent: u8) -> Move {
    m.mobility = percent;
    m
}

const fn overhead(mut m: Move) -> Move {
    m.hits_crouching = false;
    m
}

const fn unblockable(mut m: Move) -> Move {
    m.unblockable = true;
    m
}

const fn gated(mut m: Move) -> Move {
    m.needs_mechanic = true;
    m
}

// ---------------------------------------------------------------------------
// Bulwark -- committed, not slow. Wins by denying space.
// ---------------------------------------------------------------------------
pub const BULWARK: &[Move] = &[
    // Fast poke. Slightly minus on block, so it is not a free mash.
    mobile(
        mv("Bash", (4, 3, 10), 60, (3, 2), (9, 10), (14, 8), 4),
        POKE_MOBILITY,
    ),
    // The overhead. Heavily punishable if read, heavily rewarding if not.
    overhead(mv("Slam", (14, 4, 24), 170, (2, 1), (7, 5), (32, 16), 11)),
    // Beats guard outright, loses badly to dodge.
    unblockable(mv(
        "Grapple",
        (20, 3, 30),
        210,
        (11, 10),
        (8, 10),
        (40, 0),
        6,
    )),
];

// ---------------------------------------------------------------------------
// Bellator -- range bands and flow. Form multiplies everything.
// ---------------------------------------------------------------------------
pub const BELLATOR: &[Move] = &[
    mobile(
        mv("Sweep", (6, 3, 12), 65, (17, 10), (11, 10), (14, 9), 5),
        POKE_MOBILITY,
    ),
    mv("Drive", (11, 3, 20), 140, (2, 1), (1, 1), (24, 14), 9),
    overhead(mv(
        "Uppercut",
        (16, 4, 24),
        165,
        (16, 10),
        (12, 10),
        (34, 15),
        12,
    )),
];

// ---------------------------------------------------------------------------
// Shadow Reaver -- two bodies. Options are a function of the line between them.
// ---------------------------------------------------------------------------
pub const SHADOW_REAVER: &[Move] = &[
    mobile(
        mv("Slash", (5, 3, 11), 62, (14, 10), (12, 10), (14, 8), 4),
        POKE_MOBILITY,
    ),
    overhead(mv(
        "Executioner",
        (16, 4, 26),
        185,
        (17, 10),
        (13, 10),
        (34, 17),
        10,
    )),
    // Blades erupt from the shadow, so it needs one placed.
    gated(mv(
        "Guillotine",
        (9, 5, 16),
        120,
        (0, 1),
        (16, 10),
        (22, 12),
        6,
    )),
];

// ---------------------------------------------------------------------------
// Elementalist -- terrain author. Ranged, and creates its own targets.
// ---------------------------------------------------------------------------
pub const ELEMENTALIST: &[Move] = &[
    mobile(
        mv("Bolt", (7, 2, 13), 45, (4, 1), (7, 10), (14, 6), 2),
        POKE_MOBILITY,
    ),
    mv("Fissure", (13, 4, 22), 110, (7, 1), (11, 10), (27, 14), 6),
    // Detonates a structure for a much wider blast, so it wants one out.
    gated(overhead(mv(
        "Fire pillar",
        (17, 5, 24),
        175,
        (5, 1),
        (2, 1),
        (34, 16),
        8,
    ))),
];

// ---------------------------------------------------------------------------
// Blood mage -- sustain through aggression. Everything costs health.
// ---------------------------------------------------------------------------
pub const BLOOD_MAGE: &[Move] = &[
    mobile(
        mv("Rend", (6, 3, 13), 55, (3, 1), (9, 10), (16, 7), 3),
        POKE_MOBILITY,
    ),
    mv(
        "Black spike",
        (18, 4, 20),
        150,
        (5, 2),
        (16, 10),
        (26, 15),
        5,
    ),
    // Committed and directional: you cannot turn while it channels.
    unblockable(mv(
        "Reaper's debt",
        (22, 5, 28),
        230,
        (5, 2),
        (2, 1),
        (36, 0),
        9,
    )),
];

// ---------------------------------------------------------------------------
// Dual mage -- melee mage riding between two forces.
// ---------------------------------------------------------------------------
pub const DUAL_MAGE: &[Move] = &[
    mobile(
        mv("Step strike", (5, 3, 12), 58, (15, 10), (1, 1), (14, 8), 4),
        POKE_MOBILITY,
    ),
    mv("Lance", (10, 4, 18), 125, (4, 1), (8, 10), (22, 13), 6),
    // A finisher: only past the deep threshold on its own side.
    gated(overhead(mv(
        "Judgement",
        (20, 5, 26),
        215,
        (3, 1),
        (11, 5),
        (30, 18),
        10,
    ))),
];

pub const fn table(class: Class) -> &'static [Move] {
    match class {
        Class::Bulwark => BULWARK,
        Class::Bellator => BELLATOR,
        Class::ShadowReaver => SHADOW_REAVER,
        Class::Elementalist => ELEMENTALIST,
        Class::BloodMage => BLOOD_MAGE,
        Class::DualMage => DUAL_MAGE,
    }
}

pub fn get(class: Class, kind: u8) -> &'static Move {
    let t = table(class);
    &t[(kind as usize).min(t.len() - 1)]
}

/// Frame data for a move, for debug overlays and documents.
pub fn frames(class: Class, kind: u8) -> (u16, u16, u16) {
    let m = get(class, kind);
    (m.startup, m.active, m.recovery)
}
