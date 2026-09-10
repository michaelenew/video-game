//! Render interpolation.
//!
//! The simulation is locked to 60 Hz; displays are not. Without interpolation a
//! perfect 60 Hz simulation still looks choppy on a 144 Hz monitor, and it reads
//! to the player as "the game feels bad" rather than as a missing feature.
//!
//! Strictly one-directional: this reads two snapshots and produces something to
//! draw. Nothing here is ever fed back into the simulation.

use crate::fx;
use sim::World;
use sim::state::{Action, MAX_PLAYERS};

/// Beyond this distance in one tick, treat the movement as a teleport and snap
/// rather than sliding the character across the arena. Blinks and swaps are
/// real moves in several kits.
const TELEPORT_SNAP: f32 = 3.0;

#[derive(Clone, Copy, Debug)]
pub struct PlayerView {
    pub pos: [f32; 3],
    /// Unit horizontal vector.
    pub facing: [f32; 3],
    pub action: Action,
    pub frames_left: u16,
    pub health: i32,
    pub grounded: bool,
    pub crouching: bool,
    /// Horizontal speed, for locomotion posing.
    pub speed: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct Frame {
    pub players: [PlayerView; MAX_PLAYERS],
    /// Simulation frame of the newer snapshot. Deterministic, so it is safe to
    /// drive cyclic animation from.
    pub sim_frame: u32,
}

/// Blend two snapshots. `alpha` is the fraction of a tick elapsed since `cur`.
///
/// Only continuous quantities are blended. Discrete state -- the action, the
/// frame counter -- is taken from `cur` rather than interpolated, because there
/// is no meaningful halfway point between startup frame 3 and startup frame 4.
pub fn interpolate(prev: &World, cur: &World, alpha: f32) -> Frame {
    let a = alpha.clamp(0.0, 1.0);
    let mut players = [view_of(&prev.players[0], &cur.players[0], a); MAX_PLAYERS];
    for (slot, (p, c)) in players
        .iter_mut()
        .zip(prev.players.iter().zip(cur.players.iter()))
    {
        *slot = view_of(p, c, a);
    }
    Frame {
        players,
        sim_frame: cur.frame,
    }
}

fn view_of(p: &sim::state::Player, c: &sim::state::Player, a: f32) -> PlayerView {
    let from = [fx(p.pos.x), fx(p.pos.y), fx(p.pos.z)];
    let to = [fx(c.pos.x), fx(c.pos.y), fx(c.pos.z)];

    let jump = dist(from, to);
    let pos = if jump > TELEPORT_SNAP {
        to
    } else {
        [
            lerp(from[0], to[0], a),
            lerp(from[1], to[1], a),
            lerp(from[2], to[2], a),
        ]
    };

    // Normalised lerp for facing. A plain component lerp shortens the vector as
    // it rotates, which shows up as the character shrinking mid-turn.
    let facing = nlerp(
        [fx(p.facing.x), 0.0, fx(p.facing.z)],
        [fx(c.facing.x), 0.0, fx(c.facing.z)],
        a,
    );

    let speed = (fx(c.vel.x).powi(2) + fx(c.vel.z).powi(2)).sqrt();

    PlayerView {
        pos,
        facing,
        action: c.action,
        frames_left: c.action.frames_left(),
        health: c.health,
        grounded: c.grounded,
        crouching: c.crouching,
        speed,
    }
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

fn dist(a: [f32; 3], b: [f32; 3]) -> f32 {
    ((b[0] - a[0]).powi(2) + (b[1] - a[1]).powi(2) + (b[2] - a[2]).powi(2)).sqrt()
}

fn nlerp(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    let v = [
        lerp(a[0], b[0], t),
        lerp(a[1], b[1], t),
        lerp(a[2], b[2], t),
    ];
    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if len < 1e-5 {
        // Degenerate: the two facings are opposed and cancelled. Keep the
        // newer one rather than emitting a zero vector.
        b
    } else {
        [v[0] / len, v[1] / len, v[2] / len]
    }
}

/// Accumulates real time and reports how many fixed ticks to run and how far
/// between the last two the renderer should draw.
///
/// Keeping this separate from the engine's own timing means the tick cadence is
/// testable, which matters because a subtle bug here is invisible until the
/// game feels wrong on somebody else's monitor.
#[derive(Debug)]
pub struct TickClock {
    accumulator: f32,
    /// Ticks in one frame are capped so a stall does not produce a burst of
    /// simulation the player cannot react to.
    max_ticks_per_frame: u32,
}

impl TickClock {
    pub fn new() -> TickClock {
        TickClock {
            accumulator: 0.0,
            max_ticks_per_frame: 5,
        }
    }

    /// Feed real elapsed seconds; get back the number of fixed ticks to run.
    pub fn advance(&mut self, dt: f32) -> u32 {
        // Clamp pathological deltas: a breakpoint or a dragged window should
        // not fast-forward the match.
        self.accumulator += dt.clamp(0.0, 0.25);
        let mut ticks = 0;
        while self.accumulator >= crate::TICK_SECONDS && ticks < self.max_ticks_per_frame {
            self.accumulator -= crate::TICK_SECONDS;
            ticks += 1;
        }
        if ticks == self.max_ticks_per_frame {
            self.accumulator = 0.0; // drop the backlog rather than spiralling
        }
        ticks
    }

    /// How far between the previous and current snapshot to draw, in 0..1.
    pub fn alpha(&self) -> f32 {
        (self.accumulator / crate::TICK_SECONDS).clamp(0.0, 1.0)
    }
}

impl Default for TickClock {
    fn default() -> Self {
        TickClock::new()
    }
}
