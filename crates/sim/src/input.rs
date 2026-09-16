//! Player input for one tick.
//!
//! Mirrors `docs/design/controls.md`. Click means attack, WASD means move,
//! space means jump, and shift means an ability when a click comes with it or a
//! dodge when only a direction does.
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

/// One tick of input from one player.
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug, Hash)]
pub struct Input {
    pub bits: u16,
    /// Where the player is looking, in 1/65536 of a turn, measured in the
    /// horizontal plane.
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

    /// Buttons only, looking down the positive X axis, level.
    pub const fn new(bits: u16) -> Input {
        Input {
            bits,
            aim: 0,
            pitch: 0,
        }
    }

    /// Buttons and a yaw, level. Most tests want exactly this.
    pub const fn aimed(bits: u16, aim: u16) -> Input {
        Input {
            bits,
            aim,
            pitch: 0,
        }
    }

    /// Buttons and a full look direction.
    pub const fn looking_at(bits: u16, aim: u16, pitch: i16) -> Input {
        Input { bits, aim, pitch }
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
        Input {
            bits: self.bits,
            aim,
            pitch,
        }
    }

    /// Both buttons at once is its own input, per the control scheme.
    pub const fn both_clicks(self) -> bool {
        self.has(Input::LEFT) && self.has(Input::RIGHT)
    }

    pub const fn any_click(self) -> bool {
        self.bits & (Input::LEFT | Input::RIGHT | Input::MIDDLE | Input::SPECIAL) != 0
    }

    /// A click is what disambiguates shift: with one it is the committed
    /// version of that attack, with only a direction it is a dodge. The click
    /// is checked first, so a heavy thrown while walking does not come out as a
    /// dodge. See `controls.md`.
    ///
    /// This used to be "shift beats WASD when both are held", which guaranteed
    /// a move-while-casting option existed. Dodge moving onto shift retired
    /// that, and nothing has replaced what it guaranteed.
    pub const fn is_ability(self) -> bool {
        self.has(Input::SHIFT) && self.any_click()
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
