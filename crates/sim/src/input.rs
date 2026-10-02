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
//!
//! **What arrives is exact when a button decides something, and held
//! otherwise.** The sender writes the look through [`WireLook`]: exact on
//! a press, a hold, a release or a channel, and otherwise the last look it
//! sent until the hand has moved a third of a degree from it. Nothing here
//! changes for it -- the reads that see a held look (the facing, the walk, a
//! creature asking what is on your screen) cannot tell -- but a still hand no
//! longer makes every remote frame a wrong guess. See
//! `docs/design/architecture.md` §"The look on the wire".

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
/// `0` is no request and `1` is versus. A hunt sets the top bit, keeps the
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
}

impl Travel {
    pub const NONE: Travel = Travel(0);
    pub const VERSUS: Travel = Travel(1);
    const HUNT: u8 = 0x80;
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

    pub const fn destination(self) -> Option<Destination> {
        match self.0 {
            0 => None,
            1 => Some(Destination::Versus),
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

/// How far the real look may drift from the one on the wire, in the look's own
/// 1/65536 of a turn, before it is sent again: 1/1024 of a turn, about a
/// third of a degree. Yaw and pitch alike.
///
/// That is the most a cast can be off the crosshair, and only one that came
/// out of a buffer after its button was let go and the mouse kept moving --
/// about 15 cm at the fire bolt's 24 m. Every other cast is exact.
///
/// **Not an Oven knob, deliberately.** The Oven holds the rules both peers
/// share, and folds them into the checksum so a mismatch desyncs loudly. This
/// is how one peer chooses to describe its own hand; two peers with different
/// bands agree perfectly, because each sends its look and both simulate what
/// was sent.
pub const LOOK_BAND: u16 = 1 << 6;

/// **The sender's half of the look on the wire**: what it last sent, and which
/// buttons were down when it did.
///
/// The look only has to be exact on frames where it decides something, and
/// those are frames where a button is involved: a move begins on a press, a
/// charge resolves on a release, a channel aims on every frame it is held.
/// Everything else that reads the look -- the facing, the walk, a creature
/// asking what is on your screen -- cannot tell a third of a degree. So this
/// sends the real look on those frames and the last one it sent on the
/// others, until the hand has moved further than [`LOOK_BAND`] from it, and a
/// still hand stops making every remote frame a wrong guess. Nothing in the
/// simulation reads it; it writes the `aim` and `pitch` the simulation reads.
/// See `docs/design/architecture.md` §"The look on the wire".
///
/// Fed **once per simulation tick**, with the input that tick sends. Fed per
/// rendered frame instead, a release on a frame that ran no tick would be seen
/// here and never sent exact.
#[derive(Clone, Copy, Debug, Default)]
pub struct WireLook {
    /// The look last written into an input, or nothing before the first.
    sent: Option<(u16, i16)>,
    /// The buttons in `Input::PRESSES` on that input.
    bits: u16,
}

impl WireLook {
    pub const fn new() -> WireLook {
        WireLook {
            sent: None,
            bits: 0,
        }
    }

    /// The input to send this tick: `real` with its look either left exact or
    /// held at the last one sent.
    ///
    /// `aiming` is whether the local fighter is channelling in the world this
    /// peer is drawing -- a channel follows the crosshair on every frame of it,
    /// so those frames are sent exact. Asking the predicted world is fine: a
    /// wrong answer costs a third of a degree on one frame, never a desync,
    /// because whatever is sent is what both peers simulate.
    pub fn send(&mut self, real: Input, aiming: bool) -> Input {
        let buttons = real.bits & Input::PRESSES;
        let exact = aiming || buttons != 0 || buttons != self.bits;
        self.bits = buttons;
        let look = match self.sent {
            Some((aim, pitch)) if !exact && !drifted(real, aim, pitch) => (aim, pitch),
            _ => (real.aim, real.pitch),
        };
        self.sent = Some(look);
        real.looking(look.0, look.1)
    }
}

/// Whether the real look is further than the band from the one on the wire.
/// Yaw wraps, so its distance is taken the short way round; pitch does not.
fn drifted(real: Input, aim: u16, pitch: i16) -> bool {
    let yaw = (real.aim.wrapping_sub(aim) as i16).unsigned_abs();
    let tilt = (i32::from(real.pitch) - i32::from(pitch)).unsigned_abs();
    yaw > LOOK_BAND || tilt > u32::from(LOOK_BAND)
}

#[cfg(test)]
mod wire_tests {
    use super::*;

    fn at(bits: u16, aim: u16, pitch: i16) -> Input {
        Input::looking_at(bits, aim, pitch)
    }

    #[test]
    fn the_first_input_is_exact() {
        let mut w = WireLook::new();
        assert_eq!(w.send(at(0, 1234, -56), false), at(0, 1234, -56));
    }

    #[test]
    fn a_still_hand_inside_the_band_sends_the_same_look() {
        let mut w = WireLook::new();
        w.send(at(Input::W, 1000, 0), false);
        let out = w.send(at(Input::W, 1000 + LOOK_BAND, -(LOOK_BAND as i16)), false);
        assert_eq!((out.aim, out.pitch), (1000, 0));
        // The stick is not a press: it rides along untouched.
        assert_eq!(out.bits, Input::W);
    }

    #[test]
    fn past_the_band_the_real_look_goes_and_becomes_the_reference() {
        let mut w = WireLook::new();
        w.send(at(0, 1000, 0), false);
        let out = w.send(at(0, 1000 + LOOK_BAND + 1, 0), false);
        assert_eq!(out.aim, 1000 + LOOK_BAND + 1);
        let next = w.send(at(0, 1000 + LOOK_BAND + 2, 0), false);
        assert_eq!(next.aim, 1000 + LOOK_BAND + 1);
    }

    #[test]
    fn yaw_is_measured_the_short_way_round() {
        let mut w = WireLook::new();
        w.send(at(0, 10, 0), false);
        // 20 units back across zero, not 65,516 forward.
        assert_eq!(w.send(at(0, 65526, 0), false).aim, 10);
    }

    #[test]
    fn a_press_a_hold_and_a_release_are_all_exact() {
        let mut w = WireLook::new();
        w.send(at(0, 1000, 0), false);
        assert_eq!(w.send(at(Input::LEFT, 1010, 3), false).aim, 1010);
        assert_eq!(w.send(at(Input::LEFT, 1020, 3), false).aim, 1020);
        assert_eq!(w.send(at(0, 1030, 3), false).aim, 1030);
        // And after the release, the band again, from the release's look.
        assert_eq!(w.send(at(0, 1040, 3), false).aim, 1030);
    }

    #[test]
    fn a_channel_is_exact_on_every_frame() {
        let mut w = WireLook::new();
        w.send(at(0, 1000, 0), false);
        assert_eq!(w.send(at(0, 1001, 0), true).aim, 1001);
        assert_eq!(w.send(at(0, 1002, 0), true).aim, 1002);
    }

    #[test]
    fn the_wire_is_never_further_than_the_band_from_the_hand() {
        let mut w = WireLook::new();
        let mut aim = 0u16;
        let mut pitch = 0i16;
        let mut rng = 0x9e37_79b9_7f4a_7c15u64;
        for _ in 0..10_000 {
            rng ^= rng << 13;
            rng ^= rng >> 7;
            rng ^= rng << 17;
            aim = aim.wrapping_add((rng as u16) % 97).wrapping_sub(48);
            pitch = (pitch + ((rng >> 16) as i16 % 9) - 4).clamp(-15000, 15000);
            let real = at(0, aim, pitch);
            let out = w.send(real, false);
            assert!(!drifted(real, out.aim, out.pitch));
        }
    }
}
