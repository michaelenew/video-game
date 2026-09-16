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
use sim::class::Ghost;
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
    /// Which way the body is moving relative to where it is facing: `x` is
    /// strafe (positive is the character's right), `z` is forward. This is what
    /// picks between the walk, the backpedal and the two sidesteps, and it has
    /// to come from velocity rather than from the input because a shove or a
    /// dodge moves you in directions you never asked for.
    pub travel: [f32; 2],
    /// Where the body is in its stride, in cycles, wrapping. Drives the walk
    /// and run clips so feet land where the body actually is rather than on a
    /// fixed cadence.
    pub stride: f32,
    pub air_frames: u16,
    pub since_landed: u16,
    pub parried: u16,
    pub crouched_for: u16,
    /// What the current stun was when it started, so a flinch can be picked by
    /// severity rather than guessed at halfway through.
    pub stun_total: u16,
    /// Vertical speed, which is what separates rising from falling.
    pub rise: f32,
    /// How fast the character is turning, in turns per second, signed: positive
    /// is to the character's right. Derived from the two snapshots rather than
    /// stored, because a rotation between two frames *is* the turn rate and
    /// putting a second copy in the snapshot would only let it disagree.
    pub turn_rate: f32,
    /// How far above the horizon the current move was aimed, in radians.
    ///
    /// `facing` is flat, because a body turns level. This is the other half of
    /// the aim, and the renderer needs it for the same reason the simulation
    /// does: a shot that leaves along the crosshair has to *look* like it left
    /// along the crosshair, or the character is pointing one way and the
    /// attack is going another.
    pub aim_pitch: f32,
    /// How far the camera's framing has swung over to the airborne one. Blended
    /// between the two snapshots like every other continuous quantity, so the
    /// view does not step at the simulation's cadence -- see
    /// `sim::state::Player::aloft`.
    pub aloft: f32,
    /// How far the creature has turned under this fighter, in radians, and so
    /// how far their whole frame of reference has turned with it -- **the
    /// simulation's own number**, out of `sim::state::Player::carry_yaw`.
    ///
    /// The camera needs it because the simulation has already spent it: a
    /// rider's look angle is the mouse plus this, and the eye, the ray the
    /// crosshair draws out of it and the body's facing are all built from that
    /// sum. A camera left on the mouse alone draws the fighter looking off to
    /// one side of the screen.
    ///
    /// It does not reset when you step off -- resetting it would whip the view
    /// round the moment you landed -- so it is not only a thing riders have.
    pub carried: f32,
}

/// The Reaver's second body, ready to draw.
///
/// A body rather than a marker, because that is what it is: a translucent copy
/// of her with its own place, its own facing and its own animation. It is drawn
/// on a second skeleton, so what the renderer needs of it is the same handful
/// of things it needs of her.
#[derive(Clone, Copy, Debug)]
pub struct ShadowView {
    pub pos: [f32; 3],
    pub facing: [f32; 3],
    /// The move it is repeating, already lagged by the simulation. `Free` when
    /// it is copying nothing.
    pub action: Action,
    pub speed: f32,
    pub travel: [f32; 2],
    /// What it is doing with itself, and how many frames it has been doing it.
    pub doing: crate::play::Ghosting,
}

#[derive(Clone, Copy, Debug)]
pub struct Frame {
    pub players: [PlayerView; MAX_PLAYERS],
    /// The Reaver's shadow, for whichever fighter is a Reaver. `None` on the
    /// other five classes, which do not have a second body.
    pub shadows: [Option<ShadowView>; MAX_PLAYERS],
    /// Simulation frame of the newer snapshot. Deterministic, so it is safe to
    /// drive cyclic animation from.
    pub sim_frame: u32,
    /// Frames left of the between-rounds pause, when a round has ended. This
    /// is the clock the loser's collapse runs on.
    pub round_left: Option<u16>,
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
    let shadows = std::array::from_fn(|i| shadow_of(&prev.players[i], &cur.players[i], a));
    Frame {
        players,
        shadows,
        sim_frame: cur.frame,
        round_left: match cur.phase {
            sim::state::Phase::RoundOver { left, .. } => Some(left),
            sim::state::Phase::Fighting => None,
        },
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

    // Velocity in the character's own frame: forward along the facing, strafe
    // across it.
    let (fwd, side) = (facing, [facing[2], 0.0, -facing[0]]);
    let vel = [fx(c.vel.x), 0.0, fx(c.vel.z)];
    let travel = [
        vel[0] * side[0] + vel[2] * side[2],
        vel[0] * fwd[0] + vel[2] * fwd[2],
    ];

    // Signed angle from the previous facing to the current one, per second.
    let prev = [fx(p.facing.x), 0.0, fx(p.facing.z)];
    let cur = [fx(c.facing.x), 0.0, fx(c.facing.z)];
    let cross = prev[2] * cur[0] - prev[0] * cur[2];
    let dot = (prev[0] * cur[0] + prev[2] * cur[2]).clamp(-1.0, 1.0);
    let turn_rate = cross.atan2(dot) / std::f32::consts::TAU * crate::TICK_HZ;

    // The aim's vertical half, as an angle. `aim_dir` is a unit vector, so its
    // height *is* the sine of the pitch. Blended between the two snapshots
    // like every other continuous quantity, so the arm does not step.
    let pitch_of = |v: &sim::state::Player| fx(v.aim_dir().y).clamp(-1.0, 1.0).asin();
    let aim_pitch = lerp(pitch_of(p), pitch_of(c), a);
    let aloft = lerp(fx(p.aloft), fx(c.aloft), a);
    // The carried yaw, blended the **short way round**. It is kept wrapped into
    // half a turn either side, so a rider turning past the back of the animal
    // steps from just under a half to just over minus a half, and a plain lerp
    // across that would swing the whole view the long way for one frame.
    let carried = {
        let (was, now) = (fx(p.carry_yaw), fx(c.carry_yaw));
        let step = now - was;
        (was + (step - step.round()) * a) * std::f32::consts::TAU
    };

    PlayerView {
        pos,
        facing,
        action: c.action,
        frames_left: c.action.frames_left(),
        health: c.health,
        grounded: c.grounded,
        crouching: c.crouching,
        speed,
        travel,
        stride: stride_between(p.stride, c.stride, a),
        air_frames: c.air_frames,
        since_landed: c.since_landed,
        parried: c.parried,
        crouched_for: c.crouched_for,
        stun_total: c.stun_total,
        rise: fx(c.vel.y),
        turn_rate,
        aim_pitch,
        aloft,
        carried,
    }
}

/// The shadow between two snapshots.
///
/// Interpolated like the body it copies, and for the same reason: it crosses
/// the arena at thirty metres a second, so a shadow drawn on the simulation's
/// own cadence would strobe across a fast monitor.
///
/// **Its clock comes out of the simulation rather than being counted here.**
/// How far into a flight it is comes from `Ghost::Casting`'s own age, and how
/// long it has been standing still is derived from that age running past the
/// end of the throw -- both in the snapshot, so a rollback redraws the same
/// frame of the same clip.
fn shadow_of(p: &sim::state::Player, c: &sim::state::Player, a: f32) -> Option<ShadowView> {
    let (was, now) = (sim::shadow::of(p)?, sim::shadow::of(c)?);
    let from = [fx(was.pos.x), fx(was.pos.y), fx(was.pos.z)];
    let to = [fx(now.pos.x), fx(now.pos.y), fx(now.pos.z)];
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
    let facing = nlerp(
        [fx(was.facing.x), 0.0, fx(was.facing.z)],
        [fx(now.facing.x), 0.0, fx(now.facing.z)],
        a,
    );

    // Speed from the step it actually took, since a shadow carries no velocity
    // -- where it is next frame is worked out from its age, not integrated.
    let step = [to[0] - from[0], to[2] - from[2]];
    let speed = (step[0] * step[0] + step[1] * step[1]).sqrt() * crate::TICK_HZ;
    let side = [facing[2], -facing[0]];
    let travel = [
        (step[0] * side[0] + step[1] * side[1]) * crate::TICK_HZ,
        (step[0] * facing[0] + step[1] * facing[2]) * crate::TICK_HZ,
    ];

    Some(ShadowView {
        pos,
        facing,
        action: sim::shadow::echo_action(c.class, now).unwrap_or(Action::Free),
        speed,
        travel,
        doing: ghosting(now),
    })
}

/// Which animation state the shadow is in, with the frame count the clip is
/// indexed by.
fn ghosting(shadow: sim::class::Shadow) -> crate::play::Ghosting {
    use crate::play::Ghosting;
    match shadow.doing {
        Ghost::Attending => Ghosting::Attending,
        Ghost::Casting { age, .. } => Ghosting::Dashing(age),
        // Coming home has no age of its own -- it is a chase rather than a
        // flight of a fixed length -- so it holds the last frame of the dash,
        // which is the pose it should be in the whole way anyway.
        Ghost::Returning { .. } => Ghosting::Dashing(u16::MAX),
        Ghost::Waiting => Ghosting::Ready(0),
    }
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

/// Blend the stride phase across a tick, the short way round.
///
/// It has to be blended like any other continuous quantity, or the legs step at
/// 60 Hz while the body moves at the display's rate and the feet visibly
/// stutter against the ground. And it has to be blended *the short way*: the
/// phase wraps, so a step from 0.99 to 0.01 of a cycle is a fiftieth of a
/// stride forward, not almost a whole one backwards.
fn stride_between(prev: u16, cur: u16, a: f32) -> f32 {
    let delta = cur.wrapping_sub(prev) as i16 as f32;
    let phase = prev as f32 + delta * a;
    phase.rem_euclid(65536.0) / 65536.0
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
