//! World state and the tick function.
//!
//! Every field here is part of the rollback snapshot. If it affects gameplay it
//! lives in `World`; if it does not, it lives in the renderer.
//!
//! **Animation state belongs here.** Rollback re-simulates past frames, so any
//! animation the engine drives independently will pop when a rollback happens.
//! Actions are frame-data state machines, which is how fighting games are built
//! anyway.
//!
//! The move set below is a Bulwark stand-in -- a fast poke, a committed slam,
//! and the guard/parry layer from `defense.md`. Both players use it, so a
//! sandbox match is a mirror. It exists to make the frame vocabulary concrete,
//! not because the Bulwark is finished.

use crate::DT;
use crate::arena::{self, BODY_RADIUS};
use crate::fixed::Fx;
use crate::input::Input;
use crate::math::V3;

pub const MAX_PLAYERS: usize = 2;
pub type PlayerId = usize;

const GRAVITY: Fx = Fx::ratio(-30, 1);
const MOVE_SPEED: Fx = Fx::ratio(7, 1);
const GUARD_MOVE_SPEED: Fx = Fx::ratio(2, 1);
const JUMP_SPEED: Fx = Fx::ratio(9, 1);
const GROUND_Y: Fx = Fx::ZERO;

/// How fast facing rotates toward the opponent, per tick. Guarding is slower,
/// which is what makes the facing arc a real cost.
const TURN_RATE: Fx = Fx::ratio(25, 100);
const GUARD_TURN_RATE: Fx = Fx::ratio(6, 100);

/// Guard covers a frontal arc, not a bubble. Threshold is the cosine of the
/// half-angle: 0.5 is a 120-degree arc.
const GUARD_ARC_COS: Fx = Fx::ratio(1, 2);

/// Frames at the start of a guard that parry instead of blocking.
const PARRY_WINDOW: u16 = 4;
const PARRY_STAGGER: u16 = 34;

/// Move identifiers.
pub const MOVE_BASH: u8 = 0;
pub const MOVE_SLAM: u8 = 1;

struct MoveData {
    startup: u16,
    active: u16,
    recovery: u16,
    damage: i32,
    reach: Fx,
    radius: Fx,
    hitstun: u16,
    blockstun: u16,
    knockback: Fx,
}

const MOVES: [MoveData; 2] = [
    // Bash -- fast poke. Safe-ish on block.
    MoveData {
        startup: 4,
        active: 3,
        recovery: 10,
        damage: 60,
        reach: Fx::ratio(3, 2),
        radius: Fx::ratio(9, 10),
        hitstun: 14,
        blockstun: 8,
        knockback: Fx::ratio(4, 1),
    },
    // Slam -- committed. Heavily punishable, heavily rewarding.
    MoveData {
        startup: 14,
        active: 4,
        recovery: 24,
        damage: 170,
        reach: Fx::ratio(2, 1),
        radius: Fx::ratio(7, 5),
        hitstun: 26,
        blockstun: 16,
        knockback: Fx::ratio(11, 1),
    },
];

/// What a character is currently doing. Durations are frame counts, matching
/// the `startup / active / recovery` vocabulary in `ability-spec.md`.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Action {
    Free,
    Startup {
        kind: u8,
        left: u16,
    },
    Active {
        kind: u8,
        left: u16,
    },
    Recovery {
        kind: u8,
        left: u16,
    },
    /// `held` counts up. The first `PARRY_WINDOW` frames parry.
    Guard {
        held: u16,
    },
    BlockStun {
        left: u16,
    },
    HitStun {
        left: u16,
    },
    Stagger {
        left: u16,
    },
}

impl Action {
    pub const fn actionable(self) -> bool {
        matches!(self, Action::Free)
    }

    pub const fn guarding(self) -> bool {
        matches!(self, Action::Guard { .. })
    }

    /// Blocking costs a vulnerable window, so it is never a safe default.
    /// See `defense.md`.
    pub const fn stunned(self) -> bool {
        matches!(
            self,
            Action::BlockStun { .. } | Action::HitStun { .. } | Action::Stagger { .. }
        )
    }

    pub const fn tag(self) -> u32 {
        match self {
            Action::Free => 0,
            Action::Startup { .. } => 1,
            Action::Active { .. } => 2,
            Action::Recovery { .. } => 3,
            Action::Guard { .. } => 4,
            Action::BlockStun { .. } => 5,
            Action::HitStun { .. } => 6,
            Action::Stagger { .. } => 7,
        }
    }

    pub const fn frames_left(self) -> u16 {
        match self {
            Action::Free => 0,
            Action::Startup { left, .. }
            | Action::Active { left, .. }
            | Action::Recovery { left, .. }
            | Action::BlockStun { left }
            | Action::HitStun { left }
            | Action::Stagger { left } => left,
            Action::Guard { held } => held,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Player {
    pub pos: V3,
    pub vel: V3,
    /// Unit horizontal vector. Stored as a vector rather than an angle so that
    /// turning needs no trigonometry -- one less determinism risk.
    pub facing: V3,
    pub health: i32,
    pub action: Action,
    pub grounded: bool,
    /// True once the current active window has connected, so a move hits once.
    pub hit_used: bool,
    /// Class mechanic resource. Interpretation is per class.
    pub mechanic: i32,
}

impl Default for Player {
    fn default() -> Self {
        Player {
            pos: V3::ZERO,
            vel: V3::ZERO,
            facing: V3::new(Fx::ONE, Fx::ZERO, Fx::ZERO),
            health: 1000,
            action: Action::Free,
            grounded: true,
            hit_used: false,
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
        w.players[1].facing = V3::new(Fx::ONE.neg(), Fx::ZERO, Fx::ZERO);
        w
    }

    /// Advance exactly one tick. This is the whole simulation.
    ///
    /// Must stay a pure function of `(self, inputs)`: no clocks, no randomness
    /// that is not seeded from state, no iteration over unordered collections.
    pub fn advance(&mut self, inputs: [Input; MAX_PLAYERS]) {
        let opponent_pos = [self.players[1].pos, self.players[0].pos];

        for i in 0..MAX_PLAYERS {
            step_player(&mut self.players[i], inputs[i], opponent_pos[i]);
        }

        // Hit resolution after both have stepped, so neither ordering wins.
        let snapshot = self.players;
        for attacker in 0..MAX_PLAYERS {
            let defender = 1 - attacker;
            if let Some(hit) = resolve_hit(&snapshot[attacker], &snapshot[defender]) {
                apply_hit(&mut self.players[defender], hit);
                self.players[attacker].hit_used = true;
                if hit.parried {
                    self.players[attacker].action = Action::Stagger {
                        left: PARRY_STAGGER,
                    };
                }
            }
        }

        separate_bodies(&mut self.players);
        self.frame = self.frame.wrapping_add(1);
    }

    /// Cheap and portable. Used for desync detection: peers exchange checksums
    /// periodically and a mismatch means the simulations diverged.
    pub fn checksum(&self) -> u64 {
        let mut h = Fnv::new();
        h.write_u32(self.frame);
        for p in &self.players {
            for v in [p.pos, p.vel, p.facing] {
                h.write_i32(v.x.raw());
                h.write_i32(v.y.raw());
                h.write_i32(v.z.raw());
            }
            h.write_i32(p.health);
            h.write_i32(p.mechanic);
            h.write_u32(p.grounded as u32);
            h.write_u32(p.hit_used as u32);
            h.write_u32(p.action.tag());
            h.write_u32(p.action.frames_left() as u32);
            let kind = match p.action {
                Action::Startup { kind, .. }
                | Action::Active { kind, .. }
                | Action::Recovery { kind, .. } => kind,
                _ => 0,
            };
            h.write_u32(kind as u32);
        }
        h.finish()
    }
}

impl Default for World {
    fn default() -> Self {
        World::new()
    }
}

#[derive(Clone, Copy)]
struct Hit {
    damage: i32,
    hitstun: u16,
    blockstun: u16,
    knockback: Fx,
    dir: V3,
    blocked: bool,
    parried: bool,
}

fn resolve_hit(attacker: &Player, defender: &Player) -> Option<Hit> {
    let Action::Active { kind, .. } = attacker.action else {
        return None;
    };
    if attacker.hit_used {
        return None;
    }
    let m = &MOVES[kind as usize];

    let centre = attacker.pos.add(attacker.facing.scale(m.reach));
    let delta = defender.pos.sub(centre);
    if delta.flat_len().raw() > m.radius.add(BODY_RADIUS).raw() {
        return None;
    }

    // Was the defender facing the attack? Guard covers an arc, not a bubble.
    let to_attacker = attacker.pos.sub(defender.pos).normalized();
    let facing_it = defender.facing.dot(to_attacker).raw() >= GUARD_ARC_COS.raw();
    let guarding = defender.action.guarding() && facing_it;
    let parried =
        matches!(defender.action, Action::Guard { held } if held < PARRY_WINDOW) && facing_it;

    Some(Hit {
        damage: m.damage,
        hitstun: m.hitstun,
        blockstun: m.blockstun,
        knockback: m.knockback,
        dir: attacker.facing,
        blocked: guarding,
        parried,
    })
}

fn apply_hit(defender: &mut Player, hit: Hit) {
    if hit.parried {
        // The parry itself costs the defender nothing. The attacker eats the
        // stagger, which the caller applies.
        return;
    }
    if hit.blocked {
        // No chip damage. The cost of blocking is knockback plus a window
        // where you cannot act -- see defense.md.
        defender.action = Action::BlockStun {
            left: hit.blockstun,
        };
        defender.vel.x = hit.dir.x.mul(hit.knockback);
        defender.vel.z = hit.dir.z.mul(hit.knockback);
    } else {
        defender.health = (defender.health - hit.damage).max(0);
        defender.action = Action::HitStun { left: hit.hitstun };
        defender.vel.x = hit.dir.x.mul(hit.knockback);
        defender.vel.z = hit.dir.z.mul(hit.knockback);
    }
}

fn step_player(p: &mut Player, input: Input, opponent: V3) {
    // Turn toward the opponent. Slower while guarding, which is what makes the
    // facing arc cost something.
    let to_opp = opponent.sub(p.pos);
    if to_opp.flat_len().raw() > 0 {
        let target = V3::new(to_opp.x, Fx::ZERO, to_opp.z).normalized();
        let rate = if p.action.guarding() {
            GUARD_TURN_RATE
        } else {
            TURN_RATE
        };
        p.facing = p.facing.add(target.sub(p.facing).scale(rate)).normalized();
    }

    let want_guard = input.has(Input::RIGHT);

    p.action = match p.action {
        Action::Startup { kind, left } if left > 0 => Action::Startup {
            kind,
            left: left - 1,
        },
        Action::Startup { kind, .. } => Action::Active {
            kind,
            left: MOVES[kind as usize].active,
        },
        Action::Active { kind, left } if left > 0 => Action::Active {
            kind,
            left: left - 1,
        },
        Action::Active { kind, .. } => Action::Recovery {
            kind,
            left: MOVES[kind as usize].recovery,
        },
        Action::Recovery { kind, left } if left > 0 => Action::Recovery {
            kind,
            left: left - 1,
        },
        Action::Recovery { .. } => Action::Free,
        Action::BlockStun { left } if left > 0 => Action::BlockStun { left: left - 1 },
        Action::HitStun { left } if left > 0 => Action::HitStun { left: left - 1 },
        Action::Stagger { left } if left > 0 => Action::Stagger { left: left - 1 },
        Action::BlockStun { .. } | Action::HitStun { .. } | Action::Stagger { .. } => Action::Free,
        Action::Guard { held } => {
            if want_guard {
                Action::Guard {
                    held: held.saturating_add(1),
                }
            } else {
                Action::Free
            }
        }
        Action::Free => {
            if input.has(Input::LEFT) {
                let kind = if input.has(Input::SHIFT) {
                    MOVE_SLAM
                } else {
                    MOVE_BASH
                };
                p.hit_used = false;
                Action::Startup {
                    kind,
                    left: MOVES[kind as usize].startup,
                }
            } else if want_guard {
                Action::Guard { held: 0 }
            } else {
                Action::Free
            }
        }
    };

    // Horizontal movement. Free at full speed, guarding at a crawl, otherwise
    // only carried momentum from knockback.
    let (ax, az) = input.move_axis();
    if p.action.actionable() && (ax != 0 || az != 0) {
        let dir = V3::new(Fx::from_int(ax), Fx::ZERO, Fx::from_int(az)).normalized();
        p.vel.x = dir.x.mul(MOVE_SPEED);
        p.vel.z = dir.z.mul(MOVE_SPEED);
    } else if p.action.guarding() && (ax != 0 || az != 0) {
        let dir = V3::new(Fx::from_int(ax), Fx::ZERO, Fx::from_int(az)).normalized();
        p.vel.x = dir.x.mul(GUARD_MOVE_SPEED);
        p.vel.z = dir.z.mul(GUARD_MOVE_SPEED);
    } else if p.action.stunned() {
        // Knockback decays rather than stopping dead.
        p.vel.x = p.vel.x.mul(Fx::ratio(86, 100));
        p.vel.z = p.vel.z.mul(Fx::ratio(86, 100));
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

    let r = arena::resolve(p.pos, p.vel, p.grounded);
    p.pos = r.pos;
    p.vel = r.vel;
    p.grounded = r.grounded;
}

/// Bodies are solid. Push them apart symmetrically so neither index wins.
fn separate_bodies(players: &mut [Player; MAX_PLAYERS]) {
    let delta = players[1].pos.sub(players[0].pos);
    let dist = delta.flat_len();
    let min = BODY_RADIUS.add(BODY_RADIUS);
    if dist.raw() == 0 || dist.raw() >= min.raw() {
        return;
    }
    let push = min.sub(dist).mul(Fx::ratio(1, 2));
    let dir = V3::new(delta.x, Fx::ZERO, delta.z).normalized();
    players[0].pos = players[0].pos.sub(dir.scale(push));
    players[1].pos = players[1].pos.add(dir.scale(push));
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

/// Frame data for a move, for debug overlays and design docs.
pub fn move_frames(kind: u8) -> (u16, u16, u16) {
    let m = &MOVES[kind as usize];
    (m.startup, m.active, m.recovery)
}

/// Parry window length, exposed for debug overlays.
pub const fn parry_window() -> u16 {
    PARRY_WINDOW
}
