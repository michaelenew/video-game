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

use crate::fixed::Fx;

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
    /// Pitch is deliberately absent: it moves the camera but nothing in the
    /// simulation, so it stays renderer-local and off the wire.
    pub aim: u16,
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
    /// The class mechanic -- **E**. Throw the shield, change form, place the
    /// shadow, raise a structure. Not an attack, so it is not a click.
    pub const MECHANIC: u16 = 1 << 10;

    /// Buttons only, looking down the positive X axis.
    pub const fn new(bits: u16) -> Input {
        Input { bits, aim: 0 }
    }

    pub const fn aimed(bits: u16, aim: u16) -> Input {
        Input { bits, aim }
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

    pub const fn has(self, bit: u16) -> bool {
        self.bits & bit != 0
    }

    pub const fn with(self, bit: u16) -> Input {
        Input {
            bits: self.bits | bit,
            aim: self.aim,
        }
    }

    pub const fn looking(self, aim: u16) -> Input {
        Input {
            bits: self.bits,
            aim,
        }
    }

    /// Both buttons at once is its own input, per the control scheme.
    pub const fn both_clicks(self) -> bool {
        self.has(Input::LEFT) && self.has(Input::RIGHT)
    }

    pub const fn any_click(self) -> bool {
        self.bits & (Input::LEFT | Input::RIGHT | Input::SPECIAL) != 0
    }

    /// Shift beats WASD when both are held, so a move-while-casting option
    /// always exists. See `controls.md`.
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
