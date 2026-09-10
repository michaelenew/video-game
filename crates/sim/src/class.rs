//! Classes and their mechanics.
//!
//! Every class spends the same two things -- frames, and its own mechanic.
//! There is no universal resource bar, which is why the mechanic enum below is
//! also the resource enum. See `ability-spec.md`.

use crate::fixed::Fx;
use crate::math::V3;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Class {
    Bulwark,
    Bellator,
    ShadowReaver,
    Elementalist,
    BloodMage,
    DualMage,
}

pub const ALL_CLASSES: [Class; 6] = [
    Class::Bulwark,
    Class::Bellator,
    Class::ShadowReaver,
    Class::Elementalist,
    Class::BloodMage,
    Class::DualMage,
];

impl Class {
    pub const fn name(self) -> &'static str {
        match self {
            Class::Bulwark => "Bulwark",
            Class::Bellator => "Bellator",
            Class::ShadowReaver => "Shadow Reaver",
            Class::Elementalist => "Elementalist",
            Class::BloodMage => "Blood mage",
            Class::DualMage => "Dual mage",
        }
    }

    /// What its abilities spend.
    pub const fn resource(self) -> &'static str {
        match self {
            Class::Bulwark => "shield position",
            Class::Bellator => "rush charge and weapon form",
            Class::ShadowReaver => "shadow position",
            Class::Elementalist => "structure slots",
            Class::BloodMage => "health",
            Class::DualMage => "meter position",
        }
    }

    pub fn starting_mechanic(self) -> Mechanic {
        match self {
            Class::Bulwark => Mechanic::Shield(Shield::Held),
            Class::Bellator => Mechanic::Forms {
                form: Form::Sword,
                rush_charged: true,
            },
            Class::ShadowReaver => Mechanic::Shadow { at: None },
            Class::Elementalist => Mechanic::Structures([None; MAX_STRUCTURES]),
            Class::BloodMage => Mechanic::Blood,
            Class::DualMage => Mechanic::Meter { value: 0 },
        }
    }
}

// ---------------------------------------------------------------------------
// Bulwark
// ---------------------------------------------------------------------------

/// The shield has a position independent of the character. That *is* the
/// mechanic -- see `bulwark.md`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Shield {
    Held,
    Planted {
        pos: V3,
    },
    Flying {
        pos: V3,
        vel: V3,
        outbound: bool,
        travelled: Fx,
    },
}

impl Shield {
    pub const fn in_hand(self) -> bool {
        matches!(self, Shield::Held)
    }

    pub const fn world_pos(self) -> Option<V3> {
        match self {
            Shield::Held => None,
            Shield::Planted { pos } | Shield::Flying { pos, .. } => Some(pos),
        }
    }
}

// ---------------------------------------------------------------------------
// Bellator
// ---------------------------------------------------------------------------

/// Three range bands. The form multiplies every move rather than each form
/// having its own move list -- three numbers instead of three tables.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Form {
    Hammer,
    Sword,
    Spear,
}

pub const FORMS: [Form; 3] = [Form::Hammer, Form::Sword, Form::Spear];

impl Form {
    pub const fn name(self) -> &'static str {
        match self {
            Form::Hammer => "hammer",
            Form::Sword => "sword",
            Form::Spear => "spear",
        }
    }

    /// Reach, damage, and recovery multipliers. Hammer hits hardest and
    /// recovers slowest; spear reaches furthest and hits weakest.
    pub const fn modifiers(self) -> (Fx, Fx, Fx) {
        match self {
            Form::Hammer => (Fx::ratio(80, 100), Fx::ratio(135, 100), Fx::ratio(130, 100)),
            Form::Sword => (Fx::ONE, Fx::ONE, Fx::ONE),
            Form::Spear => (Fx::ratio(155, 100), Fx::ratio(78, 100), Fx::ratio(95, 100)),
        }
    }
}

// ---------------------------------------------------------------------------
// Elementalist
// ---------------------------------------------------------------------------

/// Cap on structures. Low enough that the arena stays readable in third
/// person, which matters more than the combo ceiling.
pub const MAX_STRUCTURES: usize = 3;

// ---------------------------------------------------------------------------
// Dual mage
// ---------------------------------------------------------------------------

/// Meter runs -100 (deep Dark) to +100 (deep Light). Zero is centre: most
/// options, least power.
pub const METER_MAX: i32 = 100;
/// Past this depth the finishers unlock and the burn starts.
pub const METER_DEEP: i32 = 65;
/// Health lost per tick while past the deep threshold, scaled by how far.
pub const BURN_PER_TICK_AT_MAX: i32 = 3;

// ---------------------------------------------------------------------------

/// Per-class mechanic state. Part of the rollback snapshot.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Mechanic {
    Shield(Shield),
    Forms {
        form: Form,
        rush_charged: bool,
    },
    Shadow {
        at: Option<V3>,
    },
    Structures([Option<V3>; MAX_STRUCTURES]),
    /// Health is the resource, so there is no extra state to carry.
    Blood,
    Meter {
        value: i32,
    },
}

impl Mechanic {
    /// One line for the HUD.
    pub fn summary(&self) -> alloc_free::Summary {
        alloc_free::Summary::of(self)
    }
}

/// Formatting without allocating, so the simulation stays dependency-free and
/// the HUD can render mechanic state without a String per frame.
pub mod alloc_free {
    use super::{Form, Mechanic, Shield};

    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    pub enum Summary {
        Text(&'static str),
        /// A label and a number, e.g. meter position.
        Value(&'static str, i32),
    }

    impl Summary {
        pub fn of(m: &Mechanic) -> Summary {
            match m {
                Mechanic::Shield(Shield::Held) => Summary::Text("shield: held"),
                Mechanic::Shield(Shield::Planted { .. }) => Summary::Text("shield: planted"),
                Mechanic::Shield(Shield::Flying { .. }) => Summary::Text("shield: in flight"),
                Mechanic::Forms { form, rush_charged } => {
                    Summary::Text(match (form, rush_charged) {
                        (Form::Hammer, true) => "hammer / rush ready",
                        (Form::Hammer, false) => "hammer",
                        (Form::Sword, true) => "sword / rush ready",
                        (Form::Sword, false) => "sword",
                        (Form::Spear, true) => "spear / rush ready",
                        (Form::Spear, false) => "spear",
                    })
                }
                Mechanic::Shadow { at: Some(_) } => Summary::Text("shadow: out"),
                Mechanic::Shadow { at: None } => Summary::Text("shadow: held"),
                Mechanic::Structures(slots) => Summary::Value(
                    "structures",
                    slots.iter().filter(|s| s.is_some()).count() as i32,
                ),
                Mechanic::Blood => Summary::Text("blood"),
                Mechanic::Meter { value } => Summary::Value("meter", *value),
            }
        }
    }
}
