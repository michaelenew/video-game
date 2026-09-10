//! Player input for one tick.
//!
//! Mirrors `docs/design/controls.md`. Click means attack, shift means ability,
//! WASD means move, space means you move more than you otherwise would.
//!
//! Packed into a `u16` because inputs are what cross the wire every frame, and
//! rollback sends several at once.

/// One tick of input from one player.
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug, Hash)]
pub struct Input(pub u16);

impl Input {
    pub const LEFT: u16 = 1 << 0;
    pub const RIGHT: u16 = 1 << 1;
    pub const MIDDLE: u16 = 1 << 2;
    pub const SHIFT: u16 = 1 << 3;
    pub const W: u16 = 1 << 4;
    pub const A: u16 = 1 << 5;
    pub const S: u16 = 1 << 6;
    pub const D: u16 = 1 << 7;
    pub const SPACE: u16 = 1 << 8;

    pub const fn has(self, bit: u16) -> bool {
        self.0 & bit != 0
    }

    pub const fn with(self, bit: u16) -> Input {
        Input(self.0 | bit)
    }

    /// Both buttons at once is its own input, per the control scheme.
    pub const fn both_clicks(self) -> bool {
        self.has(Input::LEFT) && self.has(Input::RIGHT)
    }

    pub const fn any_click(self) -> bool {
        self.0 & (Input::LEFT | Input::RIGHT | Input::MIDDLE) != 0
    }

    /// Shift beats WASD when both are held, so a move-while-casting option
    /// always exists. See `controls.md`.
    pub const fn is_ability(self) -> bool {
        self.has(Input::SHIFT) && self.any_click()
    }

    /// Movement axis in {-1, 0, 1} per component. `a` and `d` mirror.
    pub const fn move_axis(self) -> (i32, i32) {
        let x = (self.has(Input::D) as i32) - (self.has(Input::A) as i32);
        let z = (self.has(Input::W) as i32) - (self.has(Input::S) as i32);
        (x, z)
    }
}
