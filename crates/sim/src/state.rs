//! World state and the tick function.
//!
//! Every field here is part of the rollback snapshot. If it affects gameplay it
//! lives in `World`; if it does not, it lives in the renderer.
//!
//! **Animation state belongs here.** Rollback re-simulates past frames, so any
//! animation the engine drives independently will pop when a rollback happens.
//! Actions are frame-data state machines, which is how fighting games are built
//! anyway.

use crate::DT;
use crate::fixed::Fx;
use crate::input::Input;
use crate::math::V3;

pub const MAX_PLAYERS: usize = 2;
pub type PlayerId = usize;

const GRAVITY: Fx = Fx::ratio(-30, 1);
const MOVE_SPEED: Fx = Fx::ratio(7, 1);
const JUMP_SPEED: Fx = Fx::ratio(9, 1);
const GROUND_Y: Fx = Fx::ZERO;
const ARENA_HALF: Fx = Fx::from_int(20);

/// What a character is currently doing. Durations are frame counts, matching
/// the `startup / active / recovery` vocabulary in `ability-spec.md`.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Action {
    Free,
    Startup { kind: u8, left: u16 },
    Active { kind: u8, left: u16 },
    Recovery { kind: u8, left: u16 },
    Stunned { left: u16 },
}

impl Action {
    /// Can the player act? Recovery is committed by default; cancels are a
    /// per-class concern layered on top.
    pub const fn actionable(self) -> bool {
        matches!(self, Action::Free)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Player {
    pub pos: V3,
    pub vel: V3,
    /// Facing as turns; 1.0 is a full revolution.
    pub facing: Fx,
    pub health: i32,
    pub action: Action,
    pub grounded: bool,
    /// Class mechanic resource. Interpretation is per class -- meter position
    /// for the Dual mage, shield state for the Bulwark, and so on.
    pub mechanic: i32,
}

impl Default for Player {
    fn default() -> Self {
        Player {
            pos: V3::ZERO,
            vel: V3::ZERO,
            facing: Fx::ZERO,
            health: 1000,
            action: Action::Free,
            grounded: true,
            mechanic: 0,
        }
    }
}

/// The complete rollback snapshot.
#[derive(Clone, PartialEq, Debug)]
pub struct World {
    pub frame: u32,
    pub players: [Player; MAX_PLAYERS],
}

impl World {
    pub fn new() -> World {
        let mut w = World {
            frame: 0,
            players: [Player::default(); MAX_PLAYERS],
        };
        w.players[0].pos = V3::new(Fx::from_int(-4), GROUND_Y, Fx::ZERO);
        w.players[1].pos = V3::new(Fx::from_int(4), GROUND_Y, Fx::ZERO);
        w.players[1].facing = Fx::ratio(1, 2);
        w
    }

    /// Advance exactly one tick. This is the whole simulation.
    ///
    /// Must stay a pure function of `(self, inputs)`: no clocks, no randomness
    /// that is not seeded from state, no iteration over unordered collections.
    pub fn advance(&mut self, inputs: [Input; MAX_PLAYERS]) {
        for (player, input) in self.players.iter_mut().zip(inputs.iter()) {
            step_player(player, *input);
        }
        self.frame = self.frame.wrapping_add(1);
    }

    /// Order-independent, but cheap. Used for desync detection: peers exchange
    /// checksums periodically and a mismatch means the simulations diverged.
    pub fn checksum(&self) -> u64 {
        let mut h = Fnv::new();
        h.write_u32(self.frame);
        for p in &self.players {
            h.write_i32(p.pos.x.raw());
            h.write_i32(p.pos.y.raw());
            h.write_i32(p.pos.z.raw());
            h.write_i32(p.vel.x.raw());
            h.write_i32(p.vel.y.raw());
            h.write_i32(p.vel.z.raw());
            h.write_i32(p.facing.raw());
            h.write_i32(p.health);
            h.write_i32(p.mechanic);
            h.write_u32(p.grounded as u32);
            let (tag, kind, left) = match p.action {
                Action::Free => (0u32, 0u8, 0u16),
                Action::Startup { kind, left } => (1, kind, left),
                Action::Active { kind, left } => (2, kind, left),
                Action::Recovery { kind, left } => (3, kind, left),
                Action::Stunned { left } => (4, 0, left),
            };
            h.write_u32(tag);
            h.write_u32(kind as u32);
            h.write_u32(left as u32);
        }
        h.finish()
    }
}

impl Default for World {
    fn default() -> Self {
        World::new()
    }
}

fn step_player(p: &mut Player, input: Input) {
    // Action state machine first: it gates whether movement input is accepted.
    p.action = match p.action {
        Action::Startup { kind, left } if left > 0 => Action::Startup {
            kind,
            left: left - 1,
        },
        Action::Startup { kind, .. } => Action::Active { kind, left: 3 },
        Action::Active { kind, left } if left > 0 => Action::Active {
            kind,
            left: left - 1,
        },
        Action::Active { kind, .. } => Action::Recovery { kind, left: 12 },
        Action::Recovery { kind, left } if left > 0 => Action::Recovery {
            kind,
            left: left - 1,
        },
        Action::Recovery { .. } => Action::Free,
        Action::Stunned { left } if left > 0 => Action::Stunned { left: left - 1 },
        Action::Stunned { .. } => Action::Free,
        Action::Free => {
            if input.any_click() {
                // Placeholder: one generic committed attack. Per-class kits
                // land here once the frame data exists.
                let kind = if input.is_ability() { 1 } else { 0 };
                Action::Startup {
                    kind,
                    left: if kind == 1 { 8 } else { 4 },
                }
            } else {
                Action::Free
            }
        }
    };

    // Horizontal movement, only while free.
    let (ax, az) = input.move_axis();
    if p.action.actionable() && (ax != 0 || az != 0) {
        let dir = V3::new(Fx::from_int(ax), Fx::ZERO, Fx::from_int(az)).normalized();
        p.vel.x = dir.x.mul(MOVE_SPEED);
        p.vel.z = dir.z.mul(MOVE_SPEED);
    } else {
        p.vel.x = Fx::ZERO;
        p.vel.z = Fx::ZERO;
    }

    if input.has(Input::SPACE) && p.grounded && p.action.actionable() {
        p.vel.y = JUMP_SPEED;
        p.grounded = false;
    }

    if !p.grounded {
        p.vel.y = p.vel.y.add(GRAVITY.mul(DT));
    }

    p.pos = p.pos.add(p.vel.scale(DT));

    if p.pos.y.raw() <= GROUND_Y.raw() {
        p.pos.y = GROUND_Y;
        p.vel.y = Fx::ZERO;
        p.grounded = true;
    }

    p.pos.x = p.pos.x.clamp(ARENA_HALF.neg(), ARENA_HALF);
    p.pos.z = p.pos.z.clamp(ARENA_HALF.neg(), ARENA_HALF);
}

/// FNV-1a. Chosen because it is trivially portable and has no platform-varying
/// behaviour -- the hash itself must be deterministic or desync detection lies.
struct Fnv(u64);

impl Fnv {
    fn new() -> Fnv {
        Fnv(0xcbf2_9ce4_8422_2325)
    }
    fn write_u32(&mut self, v: u32) {
        for b in v.to_le_bytes() {
            self.0 ^= b as u64;
            self.0 = self.0.wrapping_mul(0x100_0000_01b3);
        }
    }
    fn write_i32(&mut self, v: i32) {
        self.write_u32(v as u32);
    }
    fn finish(self) -> u64 {
        self.0
    }
}
