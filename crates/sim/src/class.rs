//! Classes and their mechanics.
//!
//! Every class spends the same two things -- frames, and its own mechanic.
//! There is no universal resource bar, which is why the mechanic enum below is
//! also the resource enum. See `ability-spec.md`.

use crate::fixed::Fx;
use crate::math::V3;
use crate::oven::{self, AirField};
use crate::state::Fighters;

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

impl Class {
    /// Air stats, read straight off the Oven's per-class table.
    ///
    /// The numbers are guesses; the *spread* is the design. Bulwark is heavy:
    /// low jump, falls hard, barely steers, because committing to the air
    /// should be a real decision for the class whose whole identity is holding
    /// ground. Champion is the middleweight baseline everything else is read
    /// against. The Shadow Reaver is the most mobile thing in the air, which is
    /// what a class built on repositioning should be. The Elementalist floats
    /// but steers poorly: a caster in the air is committed to where the jump
    /// was going to take her. The Dual mage is the floatiest, and long hang
    /// time is the trade for being fragile.
    ///
    /// **This was a `const fn` full of literals until 2026-09-17, and the Oven
    /// had a per-class air table nobody read.** Those knobs were editable,
    /// bakeable and folded into the tuning hash, and dragging one did exactly
    /// nothing — the silent disagreement `crates/sim/tests/knobs.rs` exists to
    /// catch, wearing its other face. The table is the only copy now.
    pub fn mobility(self) -> Mobility {
        let raw = |f: AirField| Fx::from_raw(oven::air(self, f));
        Mobility {
            jump: raw(AirField::Jump),
            gravity: raw(AirField::Gravity),
            fall_cap: raw(AirField::FallCap),
            air_speed: raw(AirField::AirSpeed),
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

    /// Does this class's lost health turn **grey** -- reclaimable, and a
    /// weapon while it is open -- rather than simply going?
    ///
    /// The Blood mage's mechanic, in one predicate: every cost and every hit
    /// she takes moves red into grey, grey fades on its own clock, and only a
    /// drink from an essence pool turns it back. Nobody else has a grey
    /// segment, and this is what keeps it that way: the bookkeeping in
    /// `state::Player` asks here rather than checking the class by name at
    /// each of the four places health leaves a body. See
    /// `docs/design/blood-mage.md`.
    pub const fn wounds_go_grey(self) -> bool {
        matches!(self, Class::BloodMage)
    }

    pub fn starting_mechanic(self) -> Mechanic {
        match self {
            Class::Bulwark => Mechanic::Shield(Shield::Held { weight: Fx::ZERO }),
            Class::Champion => Mechanic::Forms {
                form: Form::Sword,
                rush: 0,
                recharge: 0,
                rush_vel: V3::ZERO,
                chain: 0,
                chain_left: 0,
                chain_hit: false,
                takeoff: 0,
                leap_banked: false,
                rise_used: false,
                lifted: 0,
                fury: Fury {
                    clash_by: u8::MAX,
                    ..Fury::default()
                },
                liquid: Quicksilver::AT_REST,
            },
            Class::ShadowReaver => Mechanic::Shadow(Shadow::attending(V3::ZERO, V3::ZERO)),
            Class::Elementalist => Mechanic::Structures([None; MAX_STRUCTURES]),
            Class::BloodMage => Mechanic::Blood,
            // Dark to start, which is arbitrary between two symmetric forces
            // and is not nothing: one click of the other button changes it, and
            // the bar says which one she is in from the first frame.
            Class::DualMage => Mechanic::Meter {
                dark: Fx::ZERO,
                light: Fx::ZERO,
                colour: Force::Dark,
                ascending: 0,
                jumped: false,
                hopped: false,
                dash_jump: false,
                landed: 0,
                singe: Fx::ZERO,
                veil: 0,
                warded: false,
            },
        }
    }
}

// ---------------------------------------------------------------------------
// Bulwark
// ---------------------------------------------------------------------------

/// The shield has a position independent of the character. That *is* the
/// mechanic -- see `bulwark.md`.
///
/// **Every state carries its weight** -- the blows taken on it, stored, in
/// health units (see `bulwark-v2.md`). On every variant rather than beside the
/// enum, so it travels with the object: a shield thrown with three hits in it
/// is still holding three hits where it lands. Fixed point so a slow drain
/// needs no remainder kept anywhere else.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Shield {
    Held {
        weight: Fx,
    },
    Planted {
        pos: V3,
        weight: Fx,
    },
    Flying {
        pos: V3,
        vel: V3,
        outbound: bool,
        travelled: Fx,
        weight: Fx,
        /// Has this flight already struck somebody? A shield thrown out
        /// plants where it strikes, so only the way home needs to know: a
        /// recall passes through whoever is in the way, once.
        struck: bool,
        /// **Which creatures and critters this flight has already gone
        /// through**, in a hunt. A thrown one plants on the first it meets,
        /// so only the recall reads it -- once each, and on through them.
        /// See `state::World::shields_strike_the_quarry`.
        met: Met,
    },
}

/// **What a flying shield has gone through**: a bit per critter slot and a
/// bit per creature slot. One `u16` until the slots stopped being capped by
/// it: critters below bit twelve, creatures above.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, Default)]
pub struct Met {
    pub critters: crate::critter::CritterSet,
    pub creatures: crate::bits::Bits<{ crate::bits::words_for(crate::monster::MAX_MONSTERS) }>,
}

impl Met {
    pub const NONE: Met = Met {
        critters: crate::critter::CritterSet::EMPTY,
        creatures: crate::bits::Bits::EMPTY,
    };

    /// The `u16` it was -- critters in bits nought to eleven, creatures from
    /// twelve -- when everything it holds fits there, for the checksum; all
    /// ones below the top bit when it does not, and the whole of it follows.
    pub fn compat(&self) -> u32 {
        let fits = (12..crate::critter::CritterSet::CAPACITY).all(|c| !self.critters.has(c))
            && (0..crate::monster::MAX_MONSTERS).all(|s| s < 4 || !self.creatures.has(s));
        if !fits {
            return u32::MAX >> 1;
        }
        let mut v = self.critters.words()[0] & 0xFFF;
        for s in 0..crate::monster::MAX_MONSTERS.min(4) {
            if self.creatures.has(s) {
                v |= 1 << (12 + s);
            }
        }
        v
    }
}

impl Shield {
    pub const fn in_hand(self) -> bool {
        matches!(self, Shield::Held { .. })
    }

    pub const fn world_pos(self) -> Option<V3> {
        match self {
            Shield::Held { .. } => None,
            Shield::Planted { pos, .. } | Shield::Flying { pos, .. } => Some(pos),
        }
    }

    /// What the shield is holding, in whatever state it is in.
    pub const fn weight(self) -> Fx {
        match self {
            Shield::Held { weight }
            | Shield::Planted { weight, .. }
            | Shield::Flying { weight, .. } => weight,
        }
    }

    /// The same shield, holding `weight` instead.
    pub const fn with_weight(self, weight: Fx) -> Shield {
        match self {
            Shield::Held { .. } => Shield::Held { weight },
            Shield::Planted { pos, .. } => Shield::Planted { pos, weight },
            Shield::Flying {
                pos,
                vel,
                outbound,
                travelled,
                struck,
                met,
                ..
            } => Shield::Flying {
                pos,
                vel,
                outbound,
                travelled,
                weight,
                struck,
                met,
            },
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

/// **The Champion's weapon is one weapon**: a golden, mercurial liquid that
/// winds round her forearm and, in the middle of a move, runs into the shape of
/// whichever weapon threw it. Animation clocks, and nothing else -- no rule of
/// combat reads them. They are in the snapshot because the renderer draws the
/// liquid from them, and a morph run off the renderer's own clock would replay
/// differently every time a rollback re-simulated the frame it started on
/// (`docs/design/architecture.md`). See `view::arms` and
/// `docs/design/kits/champion.md`, "The weapon you can see".
///
/// The two clocks say how solid the weapon is now; the other three are what
/// they said when the weapon was last called, so the renderer can tell what it
/// is running *from* -- a coil, a sword still half there, or a spear in full.
///
/// **Four bytes, packed**, and kept in [`Mechanic::Forms`], because the
/// snapshot was exactly at its cap: the clocks saturate as soon as they stop
/// meaning anything new. A shape forms in a handful of frames, so `forged`
/// counts to [`FORGED_MAX`](Self::FORGED_MAX); it is held and melted inside
/// a few seconds, so `rest` counts to [`REST_MAX`](Self::REST_MAX).
/// `view::arms` holds its own timings under those.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Quicksilver([u8; 4]);

// Bit layout, low to high: forged 4, rest 10, was 2 (0 none, 1 + form),
// was_forged 4, was_rest 10. Thirty bits.
const FORGED_BITS: u32 = 4;
const REST_BITS: u32 = 10;

const fn saturate(v: u16, max: u16) -> u32 {
    (if v > max { max } else { v }) as u32
}

impl Quicksilver {
    /// The most frames [`forged`](Self::forged) counts before it stops.
    pub const FORGED_MAX: u16 = (1 << FORGED_BITS) - 1;
    /// The most frames [`rest`](Self::rest) counts before it stops.
    pub const REST_MAX: u16 = (1 << REST_BITS) - 1;

    /// Nothing thrown for as long as anybody can remember: the liquid is a coil.
    pub const AT_REST: Quicksilver = Quicksilver::new(
        Self::FORGED_MAX,
        Self::REST_MAX,
        None,
        Self::FORGED_MAX,
        Self::REST_MAX,
    );

    /// From its five clocks, each saturating at its maximum.
    pub const fn new(
        forged: u16,
        rest: u16,
        was: Option<Form>,
        was_forged: u16,
        was_rest: u16,
    ) -> Quicksilver {
        let was = match was {
            None => 0,
            Some(Form::Hammer) => 1,
            Some(Form::Sword) => 2,
            Some(Form::Spear) => 3,
        };
        let bits = saturate(forged, Self::FORGED_MAX)
            | saturate(rest, Self::REST_MAX) << 4
            | was << 14
            | saturate(was_forged, Self::FORGED_MAX) << 16
            | saturate(was_rest, Self::REST_MAX) << 20;
        Quicksilver(bits.to_le_bytes())
    }

    const fn bits(self) -> u32 {
        u32::from_le_bytes(self.0)
    }

    /// Frames since the weapon was last called into a form: the start of the
    /// move that holds it.
    pub const fn forged(self) -> u16 {
        (self.bits() & 0xf) as u16
    }

    /// Frames since her last move ended. Zero while one runs.
    pub const fn rest(self) -> u16 {
        (self.bits() >> 4 & 0x3ff) as u16
    }

    /// The form it was in, or running back out of, when it was last called.
    pub const fn was(self) -> Option<Form> {
        match self.bits() >> 14 & 0x3 {
            1 => Some(Form::Hammer),
            2 => Some(Form::Sword),
            3 => Some(Form::Spear),
            _ => None,
        }
    }

    /// [`forged`](Self::forged) at that call.
    pub const fn was_forged(self) -> u16 {
        (self.bits() >> 16 & 0xf) as u16
    }

    /// [`rest`](Self::rest) at that call.
    pub const fn was_rest(self) -> u16 {
        (self.bits() >> 20 & 0x3ff) as u16
    }

    /// The four bytes, for the checksum.
    pub const fn raw(self) -> u32 {
        self.bits()
    }

    /// A move has called the weapon. `from` is the form it was in before.
    pub fn call(&mut self, from: Form) {
        *self = Quicksilver::new(0, self.rest(), Some(from), self.forged(), self.rest());
    }

    /// One frame, `moving` if a move is running at the end of it.
    pub fn tick(&mut self, moving: bool) {
        let rest = if moving {
            0
        } else {
            self.rest().saturating_add(1)
        };
        *self = Quicksilver::new(
            self.forged().saturating_add(1),
            rest,
            self.was(),
            self.was_forged(),
            self.was_rest(),
        );
    }
}

/// **What the 2026-10-12 rework added to the Champion**, kept together so the
/// rest of [`Mechanic::Forms`] reads as it did. See
/// `docs/design/exploration/0015_champion_momentum_and_the_charge.md`.
///
/// Every field is a clock or a count the simulation needs to replay a frame,
/// so it is in the snapshot and the checksum like the rest.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Fury {
    /// Damage taken during the current Rush, after the bracing. Past
    /// `tuning::rush_breaks` the Rush ends the ordinary way.
    pub braced: u16,
    /// The one blow a Rush's last frames eat whole has been eaten.
    pub eaten: bool,
    /// The shoulder has landed, or been blocked, this Rush. Once a run.
    pub shouldered: bool,
    /// The three weapon buttons as they were held last frame, as bits in
    /// button order -- so a press can be told from a hold. See `beat`.
    pub clicks: u8,
    /// **The Beat** (C2): the weapon pressed inside the impact freeze of a
    /// chain link that landed, as `clicks` bits; zero for none. It is held for
    /// him until the next link comes out, and that link is a true link.
    pub beat: u8,
    /// The move running now is a true link: it hits harder and sticks longer.
    pub true_link: bool,
    /// Weapon buttons pressed and still down since, as `clicks` bits: a press
    /// not yet spent on a move. A link whose button is in here when it starts
    /// may be held; one thrown by a button held over from the link before it
    /// may not, so holding a button down still runs the string as it always
    /// did. See `state::find_the_beat`.
    pub fresh: u8,
    /// The link running now was started by a fresh press of its button, and
    /// so held, it is that weapon's finisher.
    pub may_hold: bool,
    /// The place in the string -- 0, 1 or 2 -- the finisher running now was
    /// held from: the third's are the grand finishers.
    pub place: u8,
    /// The momentum the last finisher spent, as a percentage of a full
    /// meter: what it is worth over and above itself.
    pub spent: u8,
    /// **The Clash** (C7): who this frame's blow that met his heavy one was
    /// thrown by -- a fighter's seat, `state::QUARRY` for a creature or a
    /// critter -- or `state::NOBODY`. Settled and cleared the same frame.
    pub clash_by: u8,
    /// **Momentum** (C4): charge banked by fighting, from nought to
    /// `tuning::momentum_max`. It is on him, not on whoever he hits, and he
    /// loses it by leaving: every landed hit feeds it and re-arms
    /// `momentum_left`, and once that runs out it drains.
    pub momentum: u16,
    /// Frames the meter holds before it starts to drain.
    pub momentum_left: u8,
    /// The weapon of the last hit that landed, plus one, for the swap's
    /// extra; zero for none yet.
    pub last_weapon: u8,
}

impl Fury {
    /// Which of the three tiers the meter is in: 0 below a third, then
    /// *Heated*, *Blazing*, and 3 -- *Champion* -- full.
    pub fn tier(&self) -> u8 {
        let max = crate::tuning::momentum_max().max(1);
        let m = self.momentum as i32;
        if m >= max {
            3
        } else if m * 3 >= max * 2 {
            2
        } else if m * 3 >= max {
            1
        } else {
            0
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
    /// What a jump out of the carry takes with it, flat: the dash's own
    /// velocity, banked on arrival and bled away by the dodge's decay for
    /// each frame of the carry that passes, so pressing early keeps more. The
    /// dash itself stops dead on the shadow -- see `shadow::step_her_dash` --
    /// so this is the only place the crossing's speed survives, and it
    /// survives only into the jump.
    pub lunge: V3,
    /// A jump pressed in the last frames of the dash, before it arrived,
    /// waiting to be thrown on arrival. See `shadow::bank_dash_jump`.
    pub jump_banked: bool,
    /// Frames left of showing that a send was refused: aimed somewhere she
    /// could not walk even a step toward. The crosshair reads it. See
    /// `aim::walk_toward`.
    pub refused: u8,
    /// Shift was still down when a dash landed: it dodges again only once it
    /// has come up. The dash's tail is cut on arrival (2026-10-04), so a shift
    /// held through a short dash would otherwise throw a second dodge on the
    /// frame she lands. See `shadow::shift_spent`.
    pub shift_spent: bool,
    /// Frames left before a waiting shadow **comes home on its own**: every
    /// send starts it at `tuning::shadow_return` (2026-10-12), another send
    /// starts it over, and collecting the shadow clears it.
    pub hang: u16,
    /// She collected the shadow this frame -- the dash, standing on it --
    /// and the burst is owed. Spent by `shadow::take_collected`.
    pub collected: bool,
    /// Frames left of the forward flip out of a dash jump (2026-10-10). The
    /// renderer turns her body over by it; in the snapshot, like every other
    /// clock a picture is drawn from, so a rollback does not pop it.
    pub flip: u8,
    /// Frames left before the shadow may be sent again after a dash collected
    /// it (2026-10-18): the three moves on right click wait for it. See
    /// `tuning::send_after_dash`.
    pub rest: u16,
    /// Gallows' drive (2026-10-18): the velocity it holds, and frames left of
    /// holding it, plus the frame its speed is cut to what she keeps. A
    /// velocity rather than a target, so a change of frame does not move it.
    /// See `shadow::begin_rise`.
    pub rise: V3,
    pub rising: u8,
    /// The run of Fans thrown without a break of `tuning::fan_forget` frames,
    /// and frames left before it is forgotten (2026-10-18): each one's hang is
    /// worth `tuning::fan_falloff` of the last. See `shadow::fan_hang`.
    pub fans: u8,
    pub fan_clock: u16,
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
    Returning { struck: Fighters },
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
            lunge: V3::ZERO,
            jump_banked: false,
            refused: 0,
            shift_spent: false,
            hang: 0,
            flip: 0,
            collected: false,
            rest: 0,
            rise: V3::ZERO,
            rising: 0,
            fans: 0,
            fan_clock: 0,
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
    pub struck: Fighters,
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
    pub knock_struck: Fighters,
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
    /// Its size, as a multiple of the Oven's stone: radius and height both.
    /// One on every stone the Elementalist raises. The one solid that is not
    /// one is the Bulwark's planted shield, which joins the field as a stone
    /// sized by the weight it landed with -- see `stones::gather`.
    pub scale: Fx,
    /// Frames of **fire** left on it, or zero. Fire on a stone lights it -- a
    /// pillar cast on it, a Cinder spray bursting beside it, a burning Gale
    /// passing -- and a lit stone burns whoever stands on it and **bursts**
    /// into burning debris when anything shoves or breaks it. Earth builds
    /// the field; fire decides who may use it. See `crate::stones`.
    pub lit: u16,
    /// **Brought up into the air by the earth jump**, and not yet down again.
    ///
    /// An aloft stone left the floor a little slower than she did and falls
    /// more gently (`tuning::earth_stone_gravity`), so a straight jump comes
    /// down on to it; it starts with its top at her feet, under the floor,
    /// and the floor does not hold it back until it has come up through it.
    /// It is an ordinary stone again the frame it comes to rest.
    pub aloft: bool,
    /// **Driven back into the ground**, and shatters when it gets there --
    /// an aloft stone she earth-jumped off, or one a shot drove down out of
    /// the air. See `stones::step`, which reports where each one landed.
    pub meteor: bool,
}

// ---------------------------------------------------------------------------
// Dual mage
// ---------------------------------------------------------------------------

// Two bars, Dark and Light, each from empty to `tuning::meter_max`, and a hill
// between them: inside a band nothing moves, outside it the higher rises, the
// lower falls and she burns. The lower bar gates what her body can do. All of
// it is in `crate::dual`, and every number in it is in the Oven.

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
        /// This airtime's rising attack is spent. One per trip off the
        /// ground, and a takeoff thrown from the floor is that trip's one;
        /// back when he is on his feet and free. See `state::champion_move`.
        rise_used: bool,
        /// Frames since his feet left the floor, saturating; zero on it. A
        /// rising attack clicked in the first few (`tuning::floor_grace`) is
        /// thrown from the floor he just left. See `state::rise_from_the_floor`.
        lifted: u8,
        /// A jump pressed **during** the hammer's finisher, kept until it lands.
        ///
        /// The finisher throws whoever it hits into the air, and the decision
        /// the player is being offered is whether to go with them: declining is
        /// a knock-up and a reset, accepting is a bigger knock-up and both of
        /// you off the floor on the same frame. The press arrives during the
        /// wind-up and is spent on contact, so it sits here in between -- see
        /// `state::bank_the_leap`.
        leap_banked: bool,
        /// The rework's state: the bracing, the shoulder. See [`Fury`].
        fury: Fury,
        /// The weapon as the liquid it is between moves: when it was last
        /// called into a form, and when her last move ended. Animation only;
        /// see [`Quicksilver`]. Here rather than on the player because this
        /// variant had room in the enum and the snapshot had none.
        liquid: Quicksilver,
    },
    /// The Reaver: a second body, always somewhere. See [`Shadow`].
    Shadow(Shadow),
    Structures([Option<Structure>; MAX_STRUCTURES]),
    /// Health is the resource, so there is no extra state to carry.
    Blood,
    /// The Dual mage: **two bars**, one per force, the force she is carrying,
    /// and whether the two of them have taken her. See [`Force`],
    /// `crate::dual` and `docs/design/dual-mage.md`.
    ///
    /// The bars are fixed point rather than a count because the hill between
    /// them moves them a fraction of a unit a frame: the drift and the calm are
    /// rates per second, and a bar that could only move a whole unit at a time
    /// would either sit still or lurch.
    Meter {
        /// The Dark bar, from zero to `tuning::meter_max`.
        dark: Fx,
        /// The Light bar, the same.
        light: Fx,
        /// Which force she is carrying, set by the last auto she **threw**.
        ///
        /// It is not the same question as which bar is higher, and that is the
        /// point: she can be deep in the dark and still light, having just
        /// thrown one light auto, and her casts are light until she throws a
        /// dark one.
        ///
        /// **Always one of the two, never neither.** A vessel holding two
        /// forces is holding one of them at any moment, and the alternative --
        /// carrying nothing until the first auto -- meant her casts had no
        /// direction to push in and the first key a player pressed in a match
        /// did nothing at all.
        colour: Force,
        /// Frames of ascension left. Non-zero means both bars were goaded to
        /// the top together, the wings are out, and she is burning through it
        /// on a clock.
        ascending: u16,
        /// The second jump has been spent this airtime. Reset on landing, and
        /// never consulted while ascending, when every press is a wing beat.
        jumped: bool,
        /// The air hop has been spent this airtime: an auto thrown off the
        /// floor rises like a second jump once, and after that only carries
        /// her (2026-10-12). Reset on landing, and never consulted while
        /// ascending, when every one of them rises.
        hopped: bool,
        /// A jump pressed in the last frames of the Flurry dash's run to the
        /// disk: thrown on arrival, carrying the run's speed (2026-10-13).
        dash_jump: bool,
        /// Hits landed during this ascension. What the stagger on the way out
        /// is graduated by -- see `crate::dual::end_ascension`.
        landed: u16,
        /// The fraction of a health point the burn owes and has not yet taken.
        ///
        /// The burn is a rate per second small enough that a frame's worth is
        /// less than one health, and health is a whole number; this carries the
        /// remainder forward so a slow burn is slow rather than nothing.
        singe: Fx,
        /// Frames of the Veil left: untouchable while it runs. See
        /// `crate::dual::veiled`.
        veil: u8,
        /// Standing in one of her own sigils this frame, so a wound is cut by
        /// `sigil, wards her by`. Set by the world, which knows where the
        /// sigils are, once a frame; read by `state::Player::wound`.
        warded: bool,
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

    /// The other one.
    pub const fn other(self) -> Force {
        match self {
            Force::Dark => Force::Light,
            Force::Light => Force::Dark,
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
        /// A label and a number, e.g. how many structures stand.
        Value(&'static str, i32),
        /// The Dual mage's two bars, in whole units, and the tier the lower
        /// one holds -- an empty string for none.
        Bars {
            dark: i32,
            light: i32,
            tier: &'static str,
        },
    }

    impl Summary {
        pub fn of(m: &Mechanic) -> Summary {
            match m {
                // The weight, in whole health: what a slam would spend now.
                Mechanic::Shield(s) => Summary::Value(
                    match s {
                        Shield::Held { .. } => "shield held, weight",
                        Shield::Planted { .. } => "shield planted, weight",
                        Shield::Flying { .. } => "shield in flight, weight",
                    },
                    s.weight().to_int(),
                ),
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
                    dark,
                    light,
                    ascending,
                    ..
                } => {
                    if *ascending > 0 {
                        Summary::Text("ASCENDED")
                    } else {
                        Summary::Bars {
                            dark: dark.to_int(),
                            light: light.to_int(),
                            tier: crate::dual::tier_of_bar((*dark).min(*light)).name(),
                        }
                    }
                }
            }
        }
    }
}
