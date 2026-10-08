//! Player input for one tick.
//!
//! Mirrors `docs/design/controls.md`. Click means attack, WASD means move,
//! space means jump, and **shift means dodge and nothing else**. It used to
//! also be the attack modifier -- shift plus a click threw the committed
//! version of that attack -- and the two meanings needed the click to tell them
//! apart, which is a modifier whose meaning depends on what else your hand is
//! doing. One key, one verb.
//!
//! Buttons are packed into a `u16` because inputs are what cross the wire every
//! frame, and rollback sends several at once.
//!
//! **Aim rides along with them.** Movement and attacks resolve relative to
//! where the player is looking, which makes the look direction gameplay state,
//! not presentation -- and every peer has to agree on it bit for bit or the
//! simulations diverge. Sending it as input is what makes that free: it arrives
//! by the same path as the buttons, gets predicted and rolled back by the same
//! machinery, and the camera itself stays out of the snapshot entirely.

use crate::fixed::{Fx, cos_turns, sin_turns};
use crate::math::V3;
use crate::species::SpeciesId;

/// One tick of input from one player.
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug, Hash)]
pub struct Input {
    pub bits: u16,
    /// Where the player is looking, in 1/65536 of a turn, measured in the
    /// horizontal plane.
    ///
    /// **On the wire this is the mouse; inside a frame it is the whole look.**
    /// `state::World::advance` folds the creature's turning into it (see
    /// [`Input::turned`]) before anything reads it, so every part of the
    /// simulation that asks where a fighter is looking -- the eye, the ray out
    /// of it, the facing, the walk -- gets the same angle from the same field.
    ///
    /// Integer turns rather than radians, for two reasons. It cannot drift or
    /// wrap wrong -- every `u16` is a valid angle and adding past a full turn
    /// wraps exactly, for free. And because `Fx` is 16.16, the raw `u16` *is*
    /// the fractional part of a turn already, so `aim_turns` is a widening
    /// cast rather than a conversion with rounding to disagree about.
    ///
    pub aim: u16,
    /// How far above or below the horizon the player is looking, in the same
    /// 1/65536 of a turn, signed.
    ///
    /// It used to be renderer-local, on the grounds that it moved the camera
    /// and nothing else. That stopped being true the moment abilities started
    /// landing **where the crosshair is**: the crosshair is a line in space, a
    /// line needs two angles, and where an ability lands is as much gameplay as
    /// where you walk. So it rides along with the yaw, by the same path, and
    /// rollback predicts and corrects it with the same machinery.
    ///
    /// Signed rather than wrapped, because pitch does not wrap -- it is clamped
    /// to a little under a quarter turn either way, and a pitch that wrapped
    /// past vertical would be a camera nobody could use.
    pub pitch: i16,
    /// A request to go somewhere else: the arena picker. Zero, nearly always.
    ///
    /// **On the wire because it is the only thing both peers agree on per
    /// frame.** Changing arena or creature is a fresh fight, and a fresh fight
    /// built by one client on the frame it pressed the key would be built on a
    /// different frame by the other, or never. Sent as input, it is predicted,
    /// confirmed and rolled back like a button, and `World::advance` acts on it
    /// on the same frame on both machines. See `docs/design/arenas.md`.
    pub travel: Travel,
}

/// Where a [`Input::travel`] request goes. One byte.
///
/// `0` is no request and `1` is versus. `2`, `3` and `4` are restart, pause
/// and step. `001s cccc` is a class change: seat `s` becomes class `c`, in a
/// restart of the same fight. A hunt sets the top bit, keeps the
/// species in the low five and its **temper** in the two between
/// (`crate::temper`): `1tt sssss`. So the temper travels with the creature, on
/// the same frame, and a rollback that crosses the start of a tempered hunt
/// replays it at the same temper -- there is no second message to agree on.
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug, Hash)]
pub struct Travel(pub u8);

// Five bits of species in a hunt's travel byte: every id there is has to fit.
const _: () = assert!(crate::species::COUNT <= Travel::SPECIES as usize + 1);

/// What a [`Travel`] byte means.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Destination {
    /// Fight each other, in the proving ground.
    Versus,
    /// Hunt a creature, in its own arena, at a temper.
    Hunt(SpeciesId, u8),
    /// No creature, in a chosen arena: a jump course (`crate::course`).
    Arena(crate::arena::ArenaId),
    /// The same fight, restarted, as it is: Backspace.
    Restart,
    /// Pause, or carry on: P. Stays paused through a trip.
    Pause,
    /// Pause if running, and play exactly this frame: `]`.
    Step,
    /// The same fight, restarted, with this seat's fighter as this class.
    /// Tab, and the class pickers: on the wire for the reason every trip is,
    /// so a class can change mid-match against a person.
    Class { seat: usize, class: crate::Class },
    /// **The valley** (`crate::valley`): Hearth's square, with the journey so
    /// far kept. What a run starts in, and what `V` goes back to.
    Valley,
    /// **This creature has been beaten**, on the sender's record: what a
    /// player's trophies light the valley's waystones by. Not a trip; it sets
    /// a bit of the journey on the frame it arrives, on both machines, which
    /// is how two players who have each beaten different things share a
    /// valley that is open as far as the more travelled of them has been.
    Credit(SpeciesId),
}

impl Travel {
    pub const NONE: Travel = Travel(0);
    pub const VERSUS: Travel = Travel(1);
    /// The training controls, on the wire so they work against a person the
    /// way they work for two people at one keyboard: see [`Destination`].
    pub const RESTART: Travel = Travel(2);
    pub const PAUSE: Travel = Travel(3);
    pub const STEP: Travel = Travel(4);
    /// See [`Destination::Valley`].
    pub const VALLEY: Travel = Travel(5);
    /// `000c cccc` from eight: a creature credited, see [`Destination::Credit`].
    const CREDIT: u8 = 8;
    const HUNT: u8 = 0x80;
    /// `01aa aaaa`: no creature, in arena `a`.
    const ARENA: u8 = 0x40;
    /// `001s cccc`: seat `s` becomes class `c`.
    const CLASS: u8 = 0x20;
    const SEAT: u8 = 0x10;
    const SPECIES: u8 = 0x1F;
    const TEMPER_SHIFT: u8 = 5;
    const TEMPER: u8 = 0x03;

    /// Hunt a creature as tuned.
    pub const fn hunt(species: SpeciesId) -> Travel {
        Travel::tempered(species, 0)
    }

    /// Hunt a creature at a temper. A temper past what two bits hold is the
    /// highest they do.
    pub const fn tempered(species: SpeciesId, temper: u8) -> Travel {
        let t = if temper > Travel::TEMPER {
            Travel::TEMPER
        } else {
            temper
        };
        Travel(Travel::HUNT | t << Travel::TEMPER_SHIFT | (species.0 & Travel::SPECIES))
    }

    /// Go to an arena with nobody in it to hunt: a jump course. An id past
    /// what six bits hold is no request.
    pub const fn arena(id: crate::arena::ArenaId) -> Travel {
        if id.0 > !Travel::ARENA & !Travel::HUNT {
            Travel::NONE
        } else {
            Travel(Travel::ARENA | id.0)
        }
    }

    /// Credit a creature to the valley's journey. A species past what the
    /// byte holds is no request.
    pub const fn credit(species: SpeciesId) -> Travel {
        if species.0 as u32 + Travel::CREDIT as u32 >= Travel::CLASS as u32 {
            Travel::NONE
        } else {
            Travel(Travel::CREDIT + species.0)
        }
    }

    /// Seat `seat`'s fighter becomes `class`, in a restart of the same fight.
    pub const fn class(seat: usize, class: crate::Class) -> Travel {
        Travel(Travel::CLASS | if seat == 1 { Travel::SEAT } else { 0 } | class as u8)
    }

    pub const fn destination(self) -> Option<Destination> {
        match self.0 {
            0 => None,
            1 => Some(Destination::Versus),
            2 => Some(Destination::Restart),
            3 => Some(Destination::Pause),
            4 => Some(Destination::Step),
            5 => Some(Destination::Valley),
            b if b >= Travel::CREDIT && b < Travel::CLASS => {
                Some(Destination::Credit(SpeciesId(b - Travel::CREDIT)))
            }
            b if b & (Travel::HUNT | Travel::ARENA | Travel::CLASS) == Travel::CLASS => {
                let index = (b & 0x0F) as usize;
                if index >= crate::class::ALL_CLASSES.len() {
                    return None;
                }
                Some(Destination::Class {
                    seat: if b & Travel::SEAT != 0 { 1 } else { 0 },
                    class: crate::class::ALL_CLASSES[index],
                })
            }
            b if b & Travel::HUNT == 0 && b & Travel::ARENA != 0 => Some(Destination::Arena(
                crate::arena::ArenaId(b & !Travel::ARENA),
            )),
            b if b & Travel::HUNT != 0 => Some(Destination::Hunt(
                SpeciesId(b & Travel::SPECIES),
                (b >> Travel::TEMPER_SHIFT) & Travel::TEMPER,
            )),
            _ => None,
        }
    }
}

impl Input {
    pub const LEFT: u16 = 1 << 0;
    pub const RIGHT: u16 = 1 << 1;
    /// The class special -- **Q**. Its own button rather than a modifier on a
    /// click, because it is one of the three attacks and reading it should not
    /// require reading a modifier first.
    pub const SPECIAL: u16 = 1 << 2;
    pub const SHIFT: u16 = 1 << 3;
    pub const W: u16 = 1 << 4;
    pub const A: u16 = 1 << 5;
    pub const S: u16 = 1 << 6;
    pub const D: u16 = 1 << 7;
    pub const SPACE: u16 = 1 << 8;
    /// Crouch. Lowers your hurtbox and slows you -- the answer to a high
    /// attack, and the reason not every whiff is free.
    pub const CROUCH: u16 = 1 << 9;
    /// The class mechanic -- **E**. Throw the shield, Rush, raise a structure.
    /// Not an attack, so it is not a click -- except on the three classes whose
    /// mechanic is nothing you can press, where the key carries an ability
    /// instead: see `moves::on_e`.
    pub const MECHANIC: u16 = 1 << 10;
    /// Middle click -- the scroll wheel pressed down.
    ///
    /// A third attack button, and it exists because the Champion needs three.
    /// The class special and the class mechanic were deliberately moved *off*
    /// middle click (see `controls.md`) on the grounds that it is not a button
    /// you can find reliably mid-fight -- but that argument was about a button
    /// nobody presses often. A weapon you swing constantly is the opposite
    /// case: it is found by use rather than by aim, and the option table in
    /// `controls.md` has always counted `M` among the attack buttons. There is
    /// a keyboard stand-in for it either way.
    pub const MIDDLE: u16 = 1 << 11;
    /// The two mouse **side buttons** -- the right thumb's, and the only two
    /// buttons in the scheme that cost no finger anything. Aimed abilities
    /// want the hand that aims, and this is the hand that aims. See
    /// `docs/design/exploration/0001_control_budget.md`.
    pub const SIDE_A: u16 = 1 << 12;
    pub const SIDE_B: u16 = 1 << 13;
    /// `F` and `R`: the left index finger's two keys, one row up from `D`,
    /// which is exactly the price `Q` and `E` pay for being one row up from
    /// `A` and `W`. Unaimed things live here -- state, stance, self-casts --
    /// because this is the hand that does not aim.
    pub const KEY_F: u16 = 1 << 14;
    pub const KEY_R: u16 = 1 << 15;

    /// Every button that means something on the frame it goes down -- the
    /// attacks, the mechanic, the jump and the dodge -- as against the stick
    /// and the crouch, which only mean anything while they are held. What an
    /// impact freeze keeps for the frame after it; see `state::thaw`.
    pub const PRESSES: u16 = Input::LEFT
        | Input::RIGHT
        | Input::MIDDLE
        | Input::SPECIAL
        | Input::MECHANIC
        | Input::SPACE
        | Input::SHIFT
        | Input::SIDE_A
        | Input::SIDE_B
        | Input::KEY_F
        | Input::KEY_R;

    /// Buttons only, looking down the positive X axis, level.
    pub const fn new(bits: u16) -> Input {
        Input {
            bits,
            aim: 0,
            pitch: 0,
            travel: Travel::NONE,
        }
    }

    /// Buttons and a yaw, level. Most tests want exactly this.
    pub const fn aimed(bits: u16, aim: u16) -> Input {
        Input {
            bits,
            aim,
            pitch: 0,
            travel: Travel::NONE,
        }
    }

    /// Buttons and a full look direction.
    pub const fn looking_at(bits: u16, aim: u16, pitch: i16) -> Input {
        Input {
            bits,
            aim,
            pitch,
            travel: Travel::NONE,
        }
    }

    /// A quarter turn, as the aim unit. Handy for tests and for turning a
    /// forward vector into a rightward one.
    pub const QUARTER_TURN: u16 = 1 << 14;

    /// Aim as a fraction of a turn, ready for `sin_turns` / `cos_turns`.
    ///
    /// Exact: the `u16` is already the low 16 bits of a 16.16 value.
    pub const fn aim_turns(self) -> Fx {
        Fx::from_raw(self.aim as i32)
    }

    /// Pitch as a fraction of a turn. Negative is below the horizon.
    pub const fn pitch_turns(self) -> Fx {
        Fx::from_raw(self.pitch as i32)
    }

    /// The same look, turned by `by` of a turn.
    ///
    /// **One thing turns a look the player did not turn: the ground.** Stand on
    /// the creature and it walks a curve under you, and the whole frame of
    /// reference goes with it -- see `state::Player::carry_yaw`. Folding that
    /// into the input, once, is what keeps the camera, the ray out of it, the
    /// facing and the movement on the same angle. Adding it at some of those
    /// and not the others is a camera pointed away from the fighter, which is
    /// exactly the bug this exists to make unwriteable.
    ///
    /// Exact, and it wraps for free: `aim` is the fraction of a turn in its own
    /// right, so this is an integer addition, and `sin_turns` reads the
    /// fractional bits and nothing else.
    pub const fn turned(self, by: Fx) -> Input {
        Input {
            bits: self.bits,
            aim: self.aim.wrapping_add(by.raw() as u16),
            ..self
        }
    }

    /// The line the player is looking along, as a unit vector.
    ///
    /// **This is the aiming primitive.** Everything a player places or throws
    /// is placed or thrown along it, so there is one direction in the game
    /// rather than one per ability -- see `crate::aim`.
    pub fn look_dir(self) -> V3 {
        let pitch = self.pitch_turns();
        let flat = cos_turns(pitch);
        V3::new(
            cos_turns(self.aim_turns()).mul(flat),
            sin_turns(pitch),
            sin_turns(self.aim_turns()).mul(flat),
        )
    }

    pub const fn has(self, bit: u16) -> bool {
        self.bits & bit != 0
    }

    pub const fn with(self, bit: u16) -> Input {
        Input {
            bits: self.bits | bit,
            ..self
        }
    }

    pub const fn looking(self, aim: u16, pitch: i16) -> Input {
        Input { aim, pitch, ..self }
    }

    /// The same input, asking to go somewhere else.
    pub const fn travelling(self, to: Travel) -> Input {
        Input { travel: to, ..self }
    }

    /// Both buttons at once is its own input, per the control scheme.
    pub const fn both_clicks(self) -> bool {
        self.has(Input::LEFT) && self.has(Input::RIGHT)
    }

    pub const fn any_click(self) -> bool {
        self.bits & (Input::LEFT | Input::RIGHT | Input::MIDDLE | Input::SPECIAL) != 0
    }

    /// Movement axis in {-1, 0, 1} per component, in *stick* space: `x` is
    /// strafe, `z` is forward. Turning this into a world direction needs the
    /// aim angle -- see `state::move_dir`.
    pub const fn move_axis(self) -> (i32, i32) {
        let x = (self.has(Input::D) as i32) - (self.has(Input::A) as i32);
        let z = (self.has(Input::W) as i32) - (self.has(Input::S) as i32);
        (x, z)
    }
}
