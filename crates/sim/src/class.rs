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

/// How a class moves through the air.
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
}

const fn mobility(
    jump: (i32, i32),
    gravity: (i32, i32),
    fall: (i32, i32),
    air: (i32, i32),
) -> Mobility {
    Mobility {
        jump: Fx::ratio(jump.0, jump.1),
        gravity: Fx::ratio(gravity.0, gravity.1),
        fall_cap: Fx::ratio(fall.0, fall.1),
        air_speed: Fx::ratio(air.0, air.1),
    }
}

impl Class {
    /// Air stats. The numbers are guesses; the *spread* is the design.
    pub const fn mobility(self) -> Mobility {
        match self {
            // Heavy. Low jump, falls hard, barely steers. Committing to the air
            // should be a real decision for the class whose whole identity is
            // holding ground.
            Class::Bulwark => mobility((88, 100), (118, 100), (115, 100), (9, 10)),
            // Middleweight baseline. Everything else is read against this.
            Class::Champion => mobility((1, 1), (1, 1), (1, 1), (12, 10)),
            // The most mobile thing in the air, which is what a class built on
            // repositioning should be.
            Class::ShadowReaver => mobility((110, 100), (92, 100), (95, 100), (17, 10)),
            // Floats, but steers poorly: a caster in the air is committed to
            // where the jump was going to take them.
            Class::Elementalist => mobility((105, 100), (85, 100), (88, 100), (10, 10)),
            Class::BloodMage => mobility((1, 1), (98, 100), (1, 1), (13, 10)),
            // The floatiest. Long hang time is the trade for being fragile.
            Class::DualMage => mobility((112, 100), (80, 100), (85, 100), (14, 10)),
        }
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
            Class::Champion => "a rush charge, and which weapon is in hand",
            Class::ShadowReaver => "shadow position",
            Class::Elementalist => "structure slots",
            Class::BloodMage => "health",
            Class::DualMage => "meter position",
        }
    }

    /// Does this class hit harder when its victim cannot move?
    ///
    /// The Blood mage's damage identity, carried forward from the archive: she
    /// *naturally deals increased damage on disabled enemies*. A trait rather
    /// than a knob because it is a fact about who the class is; how much it is
    /// worth is `tuning::disabled_damage_mul`, which is a knob like everything
    /// else that decides how a fight feels.
    ///
    /// It is what makes her Grasp a setup rather than a small reward. Landing
    /// all four arms roots somebody, and the root is only worth the cost of
    /// setting up if something is waiting on the other side of it.
    pub const fn preys_on_the_disabled(self) -> bool {
        matches!(self, Class::BloodMage)
    }

    pub fn starting_mechanic(self) -> Mechanic {
        match self {
            Class::Bulwark => Mechanic::Shield(Shield::Held),
            Class::Champion => Mechanic::Forms {
                form: Form::Sword,
                rush: 0,
                recharge: 0,
                rush_vel: V3::ZERO,
                chain: 0,
                chain_left: 0,
                chain_hit: false,
                takeoff: 0,
            },
            Class::ShadowReaver => Mechanic::Shadow(Shadow::attending(V3::ZERO, V3::ZERO)),
            Class::Elementalist => Mechanic::Structures([None; MAX_STRUCTURES]),
            Class::BloodMage => Mechanic::Blood,
            // Dark to start, which is arbitrary between two symmetric forces
            // and is not nothing: one click of the other button changes it, and
            // the bar says which one she is in from the first frame.
            Class::DualMage => Mechanic::Meter {
                value: 0,
                colour: Force::Dark,
                ascending: 0,
            },
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

/// Three range bands, one per mouse button.
///
/// The form used to be a **mode**: a button cycled it and it multiplied the
/// reach, damage and recovery of a shared three-move table. That is why the
/// class read as linear -- the weapon was a number, so picking one was picking
/// a number, and the three of them threw the same three moves.
///
/// It is a **button** now. Left click is the sword, middle click the hammer,
/// right click the spear, each with its own moves on the ground, in the air
/// and out of a Rush; the form is which of them you last threw. That makes
/// this enum presentation and bookkeeping -- the HUD line, the animation, the
/// weapon in the fighter's hands -- rather than a multiplier, and the
/// differences between the weapons live where a player can feel them, in the
/// frame data and the shape of the hitbox.
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

    /// Which weapon one of the Champion's moves is thrown with.
    ///
    /// A function of the move rather than of a stored mode, which is the whole
    /// of the rebuild in one line: you cannot throw a hammer move with the
    /// spear out, because the button that threw it *was* the hammer.
    pub const fn of_move(kind: u8) -> Form {
        match crate::moves::champion::weapon(kind) {
            crate::moves::champion::HAMMER => Form::Hammer,
            crate::moves::champion::SPEAR => Form::Spear,
            _ => Form::Sword,
        }
    }
}

// ---------------------------------------------------------------------------
// Shadow Reaver
// ---------------------------------------------------------------------------

/// The Reaver's second body.
///
/// **It is never absent.** That is the whole of the mechanic: the shadow is
/// either attending her -- a translucent copy a step behind, repeating what she
/// does a beat late -- or it is out on the field, which is the same body
/// standing somewhere else. There is no third state where the class has no
/// shadow, and there used to be: `Option<V3>` said "nowhere", the class spent
/// half a match with nothing to swap to, cut with, or erupt from, and every
/// ability that reads the shadow had to carry a branch for the case where the
/// mechanic did not exist.
///
/// The position is carried in every state, including `Attending`, because the
/// attending shadow is a thing on screen with a place of its own -- it trails
/// the body rather than being welded to it -- and because the one move aimed
/// *at the mechanic* asks where the shadow is without caring which of these it
/// is doing.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Shadow {
    /// Where it is standing, live.
    pub pos: V3,
    /// Which way it is turned. Its own, because a shadow sent out keeps facing
    /// the way it was going while she turns to watch it.
    pub facing: V3,
    pub doing: Ghost,
    /// The move it is repeating, and how many frames since *she* began it.
    ///
    /// A move index and an age rather than a copy of her action, so the echo is
    /// a pure function of two numbers: rollback re-derives which phase the
    /// shadow is in rather than replaying a state machine that has to agree
    /// with hers. [`NO_ECHO`] when it is copying nothing.
    pub echo: u8,
    pub echo_age: u16,
    /// The echo has already connected, so its swing lands once -- the same rule
    /// `Player::hit_used` is for the body it is copying.
    pub echo_used: bool,
    /// Frames left of *her* dash to it.
    ///
    /// On the shadow rather than on the fighter, because it is the only thing
    /// in the game that reads it and because the pair of them is one mechanic:
    /// a dash to the shadow is a fact about where the shadow is. It rides
    /// alongside `Action::Dodge`, which supplies the invulnerability -- the
    /// dash is the dodge, aimed.
    pub dash: u16,
    /// Frames left of **the carry**: the window that opens the moment a dash
    /// arrives, while the speed it crossed at is still under her.
    ///
    /// It is there to be jumped out of. The dash ends at thirty-four metres a
    /// second and the dodge's tail bleeds that away over a few frames; a jump
    /// pressed inside the window takes what is left of it up with her instead
    /// of throwing it on the floor. Pressing early keeps more, so the tech has
    /// a gradient rather than a pass mark -- see `tuning::shadow_carry`.
    pub carry: u16,
}

/// [`Shadow::echo`] when the shadow is not repeating anything.
pub const NO_ECHO: u8 = u8::MAX;

/// What the shadow is doing with itself, as opposed to what it is copying.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Ghost {
    /// With her. It trails the body by `shadow_trail` metres and eases toward
    /// that mark rather than being pinned to it, which is what makes it read as
    /// something following her rather than a decal.
    Attending,
    /// Racing out to the spot the crosshair picked, and nothing can stop it
    /// short -- it is a shadow.
    ///
    /// Progress is `age` against `shadow_send_frames`, so the whole flight is a
    /// function of the frame it left on, the same rule `crate::effects` follows
    /// and for the same reason: a rollback that lands in the middle of one
    /// reproduces it exactly instead of re-integrating it.
    Casting { from: V3, to: V3, age: u16 },
    /// Standing where it arrived. The state everything in the kit is set up
    /// for.
    Waiting,
    /// Dashing home, cutting and slowing whatever it passes through.
    ///
    /// One bit per fighter it has already caught, so a fighter it runs over is
    /// hit once by the return rather than once a frame.
    Returning { struck: u8 },
}

impl Shadow {
    /// The shadow of a Reaver standing at `pos`, attending her.
    pub fn attending(pos: V3, facing: V3) -> Shadow {
        Shadow {
            pos,
            facing,
            doing: Ghost::Attending,
            echo: NO_ECHO,
            echo_age: 0,
            echo_used: false,
            dash: 0,
            carry: 0,
        }
    }

    /// Is it out on the field rather than at her shoulder?
    pub const fn is_out(self) -> bool {
        !matches!(self.doing, Ghost::Attending)
    }

    /// Is it standing still out there -- the state the kit is built around?
    pub const fn is_waiting(self) -> bool {
        matches!(self.doing, Ghost::Waiting)
    }

    /// Is it on its way home, cutting as it comes?
    pub const fn is_returning(self) -> bool {
        matches!(self.doing, Ghost::Returning { .. })
    }
}

// ---------------------------------------------------------------------------
// Elementalist
// ---------------------------------------------------------------------------

/// Cap on structures. Low enough that the arena stays readable in third
/// person, which matters more than the combo ceiling.
pub const MAX_STRUCTURES: usize = 3;

/// One raised structure -- a stone.
///
/// State only. What a stone *does* -- fall, shove, lift, erupt, hold a fighter
/// up -- lives in [`crate::stones`], because it is the only thing in the game
/// that is both a solid and part of the snapshot.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Structure {
    /// The base of the column: the footprint it stands on.
    ///
    /// A live position rather than the spot it was cast at. Stones move: one
    /// erupting underneath throws this one off the top, and it lands wherever
    /// that leaves it.
    pub at: V3,
    /// How it is moving. Zero for a stone at rest, which is most of them most
    /// of the time.
    pub vel: V3,
    /// Frames since it was raised, saturating.
    ///
    /// It drives the rise out of the ground and **nothing else** -- structures
    /// have no lifetime, and the last time they had one the Elementalist's
    /// special quietly stopped working. An age is not a clock: this one stops
    /// counting and the structure stays until a fourth is raised.
    pub age: u16,
    /// One bit per fighter the eruption has already caught.
    ///
    /// A stone erupts once, so it can catch you once -- the same rule as
    /// `Player::hit_used`, kept per victim because a stone is not aimed at
    /// anybody in particular.
    pub struck: u8,
    /// Mid-flight from the Elementalist's own auto, aimed through it rather
    /// than at a fighter. See `docs/design/kits/elementalist.md`.
    ///
    /// A separate flag rather than inferring it from velocity: a stone
    /// knocked into another by an eruption is also moving fast, and that
    /// knock must not be mistaken for a kick that hurts people.
    pub launched: bool,
    /// Where the current launch began, so `crate::stones` can tell how far
    /// through its travel the stone is and ease its speed off near the end.
    pub launch_from: V3,
    /// Which fighters *this* launch has already hit.
    ///
    /// Kept apart from `struck`: a stone erupts once ever, but it can be
    /// kicked again and again, and each kick is a fresh event that ought to
    /// be able to hurt someone the last one already caught.
    pub knock_struck: u8,
    /// How many frames this one takes to come out of the floor.
    ///
    /// Per stone rather than a single rule, because there are two ways a stone
    /// arrives and they are not the same event. **Raised**, the rise *is* the
    /// telegraph — `tuning::structure_rise`, a quarter of a second of ground
    /// churning before anything hurts. **Driven up** by Landfall, the telegraph
    /// was the plunge that put it there, and the slab comes out of the floor in
    /// a fraction of the time. Both still read the same curve, so the churn and
    /// the eruption move with whichever number applies.
    pub rise: u16,
    /// The direction this stone's eruption throws whoever it catches, as a unit
    /// vector. [`V3::ZERO`] on an ordinary stone, which erupts straight up and
    /// leaves you standing where you were.
    ///
    /// One stone in the game has one: Landfall's, levered out of the ground at
    /// an angle away from the Elementalist, so the slab shoves along the angle
    /// rather than merely appearing. The direction is stored rather than worked
    /// out at the moment it erupts because by then she has landed and moved on,
    /// and a push aimed from where she is *now* would point somewhere nobody
    /// chose. See `crate::stones` and `tuning::landfall_tilt`.
    pub erupt: V3,
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
    /// The Champion: which weapon was last swung, and where Rush is.
    ///
    /// Rush is one stored charge that dashes and, spent from a recovery,
    /// cancels it -- so the whole of it is "how many frames of dash are left"
    /// and "how long until I get another one". The velocity rides along
    /// because a dash has to hold its line: facing follows the mouse while you
    /// are free to act, and a dash steered by the mouse is a turn, not a dash.
    Forms {
        form: Form,
        /// Frames of dash left. Non-zero means the Champion is rushing, which
        /// is what makes the mouse buttons throw the Rush moves.
        rush: u16,
        /// Frames until the charge is back. Zero is ready.
        recharge: u16,
        /// The horizontal velocity the dash drives, in metres per second.
        rush_vel: V3,
        /// How many hits of the ground chain are already behind you: 0 means
        /// the next click opens, 2 means the next click finishes.
        ///
        /// **A number, not a list of moves.** Which weapon each hit was thrown
        /// with is not remembered anywhere, and that is the mechanic rather
        /// than an omission: every hit is a free choice of all three weapons,
        /// so the only thing the chain has to carry is how deep it is. See
        /// `moves::champion::link`.
        chain: u8,
        /// Frames left in which the chain survives. Zero means the next hit
        /// opens a new one.
        ///
        /// A clock rather than a flag because a chain is a rhythm: it has to
        /// outlive the frames between one swing ending and the next beginning,
        /// and it has to die if the player stops. Re-armed on every link.
        chain_left: u16,
        /// The last link connected, so its recovery may be cut short to make
        /// room for the next one. Cleared when the next link starts.
        ///
        /// **The chain is a hit confirm.** Blocked and whiffed links pay their
        /// whole recovery, which is what keeps every move's printed frame data
        /// true against a defender who did something about it -- see
        /// `state::chain_cancel`.
        chain_hit: bool,
        /// Frames left in which a weapon click is a **takeoff** rather than the
        /// grounded or airborne move.
        ///
        /// Armed by the jump button and spent by the click, so "attack on the
        /// same press as jump" survives the two arriving a few frames apart --
        /// which they always do. Zero everywhere else.
        takeoff: u16,
    },
    /// The Reaver: a second body, always somewhere. See [`Shadow`].
    Shadow(Shadow),
    Structures([Option<Structure>; MAX_STRUCTURES]),
    /// Health is the resource, so there is no extra state to carry.
    Blood,
    /// The Dual mage: where she sits between the two forces, which of them she
    /// is currently *in*, and whether she has been driven off the end of the
    /// bar. See [`Force`] and `docs/design/dual-mage.md`.
    Meter {
        value: i32,
        /// Which force she is carrying, set by the last auto she **threw**.
        ///
        /// It is not the same question as which side of the bar she is on, and
        /// that is the point: she can be deep in the dark and still light,
        /// having just thrown one light auto, and her casts are light until she
        /// throws a dark one.
        ///
        /// **Always one of the two, never neither.** A vessel holding two
        /// forces is holding one of them at any moment, and the alternative --
        /// carrying nothing until the first auto -- meant her casts had no
        /// direction to push in and the first key a player pressed in a match
        /// did nothing at all.
        colour: Force,
        /// Frames of ascension left. Non-zero means the bar was driven to an
        /// end and she is burning through it on a clock.
        ascending: u16,
    },
}

/// One of the two forces the Dual mage holds.
///
/// Named by what the class calls them rather than by left and right, because
/// the arm is how you *throw* one and this is which one you are carrying: the
/// last auto to land decides it, and every ability takes its form from it.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Force {
    Dark,
    Light,
}

impl Force {
    pub const fn name(self) -> &'static str {
        match self {
            Force::Dark => "dark",
            Force::Light => "light",
        }
    }

    /// Which way along the bar this force pulls: dark is negative, light
    /// positive, and the bar's centre is zero.
    pub const fn along(self) -> i32 {
        match self {
            Force::Dark => -1,
            Force::Light => 1,
        }
    }
}

impl Mechanic {
    /// Where this mechanic is standing in the world, if anywhere.
    ///
    /// A thrown shield, a placed shadow, the oldest structure. Held, toggled or
    /// counted mechanics — a shield in hand, a weapon form, a meter — are
    /// nowhere, and answer `None`.
    ///
    /// Exists because one move is aimed *at the mechanic* rather than at the
    /// crosshair: see [`crate::aim::mechanic_path`].
    pub fn placed(&self) -> Option<V3> {
        match self {
            Mechanic::Shield(s) => s.world_pos(),
            // Always somewhere: at her shoulder, or out on the field. The
            // Guillotine erupts wherever that is.
            Mechanic::Shadow(shadow) => Some(shadow.pos),
            Mechanic::Structures(slots) => slots.iter().flatten().next().map(|s| s.at),
            Mechanic::Forms { .. } | Mechanic::Blood | Mechanic::Meter { .. } => None,
        }
    }

    /// One line for the HUD.
    pub fn summary(&self) -> alloc_free::Summary {
        alloc_free::Summary::of(self)
    }
}

/// Formatting without allocating, so the simulation stays dependency-free and
/// the HUD can render mechanic state without a String per frame.
pub mod alloc_free {
    use super::{Form, Ghost, Mechanic, Shield};

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
                // Three things want saying and there is one line to say them
                // in, so they are ranked by how soon they stop being true. A
                // dash lasts twenty frames, a live chain a little longer, and
                // the charge is either back or it is not -- so the dash wins,
                // then which hit comes next, and the charge is what the line
                // says when nothing is happening.
                Mechanic::Forms {
                    form,
                    rush,
                    recharge,
                    chain,
                    chain_left,
                    ..
                } => Summary::Text(if *rush > 0 {
                    "RUSHING"
                } else if *chain_left > 0 && *chain > 0 {
                    match (form, *chain) {
                        (Form::Hammer, 1) => "hammer / 2nd hit",
                        (Form::Hammer, _) => "hammer / 3rd hit",
                        (Form::Sword, 1) => "sword / 2nd hit",
                        (Form::Sword, _) => "sword / 3rd hit",
                        (Form::Spear, 1) => "spear / 2nd hit",
                        (Form::Spear, _) => "spear / 3rd hit",
                    }
                } else {
                    match (form, *recharge == 0) {
                        (Form::Hammer, true) => "hammer / rush ready",
                        (Form::Hammer, false) => "hammer",
                        (Form::Sword, true) => "sword / rush ready",
                        (Form::Sword, false) => "sword",
                        (Form::Spear, true) => "spear / rush ready",
                        (Form::Spear, false) => "spear",
                    }
                }),
                Mechanic::Shadow(shadow) => Summary::Text(match shadow.doing {
                    Ghost::Attending => "shadow: with you",
                    Ghost::Casting { .. } => "shadow: going out",
                    Ghost::Waiting => "shadow: out",
                    Ghost::Returning { .. } => "shadow: coming back",
                }),
                Mechanic::Structures(slots) => Summary::Value(
                    "structures",
                    slots.iter().filter(|s| s.is_some()).count() as i32,
                ),
                Mechanic::Blood => Summary::Text("blood"),
                Mechanic::Meter {
                    value, ascending, ..
                } => {
                    if *ascending > 0 {
                        Summary::Text("ASCENDED")
                    } else {
                        Summary::Value("meter", *value)
                    }
                }
            }
        }
    }
}
