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
    Champion,
    ShadowReaver,
    Elementalist,
    BloodMage,
    DualMage,
}

pub const ALL_CLASSES: [Class; 6] = [
    Class::Bulwark,
    Class::Champion,
    Class::ShadowReaver,
    Class::Elementalist,
    Class::BloodMage,
    Class::DualMage,
];

/// How a class moves through the air, and how far it goes when it is hit.
///
/// Classes differ in the air before they differ anywhere else, the way they do
/// in a platform fighter. Weight is the most legible difference a character can
/// have: you can read it from across the arena in the first second of a match,
/// before you know a single one of their moves.
///
/// Everything here is a multiplier on the universal constant in `tuning.rs`
/// except `air_speed`, which is absolute because it is the Quake wish-speed cap
/// and is not meaningfully "a fraction of walking".
#[derive(Clone, Copy, Debug)]
pub struct Mobility {
    /// Multiplier on takeoff speed.
    pub jump: Fx,
    /// Multiplier on gravity. Above one is a fast-faller.
    pub gravity: Fx,
    /// Multiplier on terminal velocity.
    pub fall_cap: Fx,
    /// The air-control budget: how much speed a strafe may add per burst.
    /// Small next to the walk speed on purpose -- it turns you, it does not
    /// carry you.
    pub air_speed: Fx,
    /// What a fighter divides incoming knockback by. Above one is a heavy.
    ///
    /// Knockback only. Hitstun is left alone deliberately: it is authored per
    /// move here rather than derived from knockback the way Smash derives it,
    /// and a class-dependent hitstun would mean the frame table's "on hit"
    /// column stopped being a property of the move. The cost of that choice is
    /// the interesting part -- a heavy takes the same stun and travels less far
    /// out of it, so heavies are combo food. That is the right answer arrived
    /// at honestly, and it is the same answer a platform fighter gives.
    pub weight: Fx,
}

impl Class {
    /// Air stats and weight, live from the Oven.
    ///
    /// These used to be `const` ratios written here. They are exactly as much
    /// a feel number as any frame count -- a class's weight decides whether it
    /// can be comboed at all -- so they are knobs like everything else, and the
    /// prose that used to justify each value now sits beside the classes it
    /// describes rather than beside the digits.
    ///
    /// The numbers are guesses; the *spread* is the design.
    ///
    /// - **Bulwark** is the heavy: low jump, falls hard, barely steers, and
    ///   does not travel when struck. Committing to the air should be a real
    ///   decision for the class whose identity is holding ground.
    /// - **Champion** is the middleweight baseline. Everything is read against
    ///   it.
    /// - **Shadow Reaver** is the most mobile thing in the air, which is what a
    ///   class built on repositioning should be.
    /// - **Elementalist** floats but steers poorly: a caster in the air is
    ///   committed to wherever the jump was taking them.
    /// - **Blood mage** is slightly heavy, which matters more than it looks --
    ///   it spends its own health, and spending health is what makes you
    ///   launchable. The weight is the counterweight to its own mechanic.
    /// - **Dual mage** is the floatiest and the lightest. Long hang time and a
    ///   long ride off every hit are the trade for the kit.
    pub fn mobility(self) -> Mobility {
        use crate::oven::{self, AirField as A};
        let raw = |f: A| Fx::from_raw(oven::air(self, f));
        Mobility {
            jump: raw(A::Jump),
            gravity: raw(A::Gravity),
            fall_cap: raw(A::FallCap),
            air_speed: raw(A::AirSpeed),
            weight: raw(A::Weight),
        }
    }

    /// What incoming knockback is divided by. Above one is a heavy.
    pub fn weight(self) -> Fx {
        self.mobility().weight
    }

    pub const fn name(self) -> &'static str {
        match self {
            Class::Bulwark => "Bulwark",
            Class::Champion => "Champion",
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
            Class::Champion => "rush charge and weapon form",
            Class::ShadowReaver => "shadow position",
            Class::Elementalist => "structure slots",
            Class::BloodMage => "health",
            Class::DualMage => "meter position",
        }
    }

    pub fn starting_mechanic(self) -> Mechanic {
        match self {
            Class::Bulwark => Mechanic::Shield(Shield::Held),
            Class::Champion => Mechanic::Forms {
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
// Champion
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
    ///
    /// Nine numbers that are most of the class -- three kits out of one move
    /// table -- so they live in the Oven with everything else that decides how
    /// a fighter feels.
    pub fn modifiers(self) -> (Fx, Fx, Fx) {
        crate::tuning::form_modifiers(self)
    }
}

// ---------------------------------------------------------------------------
// Elementalist
// ---------------------------------------------------------------------------

/// Cap on structures. Low enough that the arena stays readable in third
/// person, which matters more than the combo ceiling.
pub const MAX_STRUCTURES: usize = 3;

/// One raised structure.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Structure {
    pub at: V3,
    /// Frames since it was raised, saturating.
    ///
    /// It drives the rise out of the ground and **nothing else** -- structures
    /// have no lifetime, and the last time they had one the Elementalist's
    /// special quietly stopped working. An age is not a clock: this one stops
    /// counting and the structure stays until a fourth is raised.
    pub age: u16,
}

// ---------------------------------------------------------------------------
// Dual mage
// ---------------------------------------------------------------------------

// The meter runs from deep Dark to deep Light, with zero at the centre: most
// options, least power. Its range, the depth at which finishers unlock and the
// burn starts, and how hard it burns are all in the Oven -- see
// `tuning::meter_max`, `meter_deep` and `meter_burn`.

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
    Structures([Option<Structure>; MAX_STRUCTURES]),
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
