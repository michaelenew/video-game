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
//! Feel numbers live in [`crate::tuning`]; move tables live in
//! [`crate::moves`]. Neither belongs inline here — they change constantly and
//! need to be findable. See `docs/design/feel-log.md`.

use crate::DT;
use crate::arena;
pub use crate::class::Shield;
use crate::class::{self, BURN_PER_TICK_AT_MAX, Class, Form, METER_DEEP, METER_MAX, Mechanic};
use crate::fixed::Fx;
use crate::input::Input;
use crate::math::V3;
use crate::moves;
use crate::tuning as t;

pub const MAX_PLAYERS: usize = 2;
pub type PlayerId = usize;

const GROUND_Y: Fx = Fx::ZERO;

/// Move slots are positional and mean the same thing on every class, which is
/// what lets one control scheme drive six kits. See `controls.md`.
pub const SLOT_POKE: u8 = 0;
pub const SLOT_COMMITTED: u8 = 1;
pub const SLOT_SPECIAL: u8 = 2;
pub use crate::tuning::MAX_HEALTH;

/// Shield flight, Bulwark only.
const SHIELD_SPEED: Fx = Fx::ratio(19, 1);
const SHIELD_RANGE: Fx = Fx::from_int(9);
const SHIELD_DAMAGE: i32 = 85;
const SHIELD_RADIUS: Fx = Fx::ratio(45, 100);
const LEAP_SPEED: Fx = Fx::ratio(15, 1);

/// Shadow leash, Reaver only. Past this the shadow snaps back.
const SHADOW_LEASH: Fx = Fx::from_int(8);

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
    /// `held` counts up. The first `t::PARRY_WINDOW` frames parry.
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
    /// Committed evasive roll. Invulnerable for its opening frames, then
    /// recovering and vulnerable -- so it beats a read and loses to a delayed
    /// attack.
    Dodge {
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

    /// Invulnerable frames of a dodge. Nothing else grants invulnerability.
    pub const fn invulnerable(self) -> bool {
        matches!(self, Action::Dodge { left } if left + t::DODGE_IFRAMES > t::DODGE_FRAMES)
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
            Action::Dodge { .. } => 8,
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
            | Action::Stagger { left }
            | Action::Dodge { left } => left,
            Action::Guard { held } => held,
        }
    }
}

/// Match flow.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Phase {
    Fighting,
    /// `winner` is a player index, or `u8::MAX` for a double knockout.
    RoundOver {
        winner: u8,
        left: u16,
    },
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
    pub class: Class,
    pub mechanic: Mechanic,
    pub rounds_won: u8,
    pub crouching: bool,
}

impl Player {
    pub fn new(class: Class) -> Player {
        Player {
            class,
            mechanic: class.starting_mechanic(),
            ..Player::default()
        }
    }

    /// The Bulwark's shield, if this is one.
    pub fn shield(&self) -> Option<Shield> {
        match self.mechanic {
            Mechanic::Shield(s) => Some(s),
            _ => None,
        }
    }

    /// Can this class act right now given its mechanic? Gated moves need the
    /// mechanic in a particular state; ungated ones never do.
    pub fn mechanic_ready(&self, kind: u8) -> bool {
        if !moves::get(self.class, kind).needs_mechanic {
            return match self.mechanic {
                // The Bulwark's ordinary moves need the shield in hand.
                Mechanic::Shield(sh) => sh.in_hand(),
                _ => true,
            };
        }
        match self.mechanic {
            Mechanic::Shield(sh) => sh.in_hand(),
            Mechanic::Shadow { at } => at.is_some(),
            Mechanic::Structures(slots) => slots.iter().any(|s| s.is_some()),
            Mechanic::Meter { value } => value.abs() >= METER_DEEP,
            Mechanic::Forms { .. } | Mechanic::Blood => true,
        }
    }

    /// Height of the hurtbox. Crouching ducks under anything aimed high.
    pub fn hurt_height(&self) -> Fx {
        if self.crouching {
            arena::BODY_HEIGHT.mul(t::CROUCH_HEIGHT_SCALE)
        } else {
            arena::BODY_HEIGHT
        }
    }
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
            class: Class::Bulwark,
            mechanic: Mechanic::Shield(Shield::Held),
            rounds_won: 0,
            crouching: false,
        }
    }
}

/// The complete rollback snapshot.
#[derive(Clone, PartialEq, Debug)]
pub struct World {
    pub frame: u32,
    pub players: [Player; MAX_PLAYERS],
    pub phase: Phase,
}

impl World {
    pub fn new() -> World {
        World::with_classes([Class::Bulwark; MAX_PLAYERS])
    }

    /// Start a match with a chosen class per side.
    pub fn with_classes(classes: [Class; MAX_PLAYERS]) -> World {
        let mut w = World {
            frame: 0,
            players: [Player::default(); MAX_PLAYERS],
            phase: Phase::Fighting,
        };
        for (p, class) in w.players.iter_mut().zip(classes.iter()) {
            *p = Player::new(*class);
        }
        w.reset_positions();
        w
    }

    /// Put both fighters back on their marks. Keeps round wins and classes.
    fn reset_positions(&mut self) {
        for (i, p) in self.players.iter_mut().enumerate() {
            let wins = p.rounds_won;
            let class = p.class;
            let side = if i == 0 { -4 } else { 4 };
            *p = Player {
                pos: V3::new(Fx::from_int(side), GROUND_Y, Fx::ZERO),
                facing: V3::new(
                    if i == 0 { Fx::ONE } else { Fx::ONE.neg() },
                    Fx::ZERO,
                    Fx::ZERO,
                ),
                rounds_won: wins,
                ..Player::new(class)
            };
        }
    }

    /// Advance exactly one tick. This is the whole simulation.
    ///
    /// Must stay a pure function of `(self, inputs)`: no clocks, no randomness
    /// that is not seeded from state, no iteration over unordered collections.
    pub fn advance(&mut self, inputs: [Input; MAX_PLAYERS]) {
        self.frame = self.frame.wrapping_add(1);

        if let Phase::RoundOver { winner, left } = self.phase {
            if left > 0 {
                self.phase = Phase::RoundOver {
                    winner,
                    left: left - 1,
                };
                // Bodies still settle during the pause; nothing else acts.
                for p in self.players.iter_mut() {
                    settle(p);
                }
                return;
            }
            self.reset_positions();
            self.phase = Phase::Fighting;
            return;
        }

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
                        left: t::PARRY_STAGGER,
                    };
                }
            }
        }

        // A thrown shield is its own threat while it travels.
        let shields = [self.players[0].shield(), self.players[1].shield()];
        for (owner, shield) in shields.iter().enumerate() {
            let target = 1 - owner;
            let Some(Shield::Flying { pos, .. }) = *shield else {
                continue;
            };
            {
                let victim = self.players[target];
                let d = victim.pos.sub(pos);
                let hit_range = SHIELD_RADIUS.add(t::BODY_RADIUS);
                let vertical = d.y.abs().raw() < arena::BODY_HEIGHT.raw();
                if vertical && d.flat_len().raw() < hit_range.raw() && !victim.action.invulnerable()
                {
                    let dir = V3::new(d.x, Fx::ZERO, d.z).normalized();
                    apply_hit(
                        &mut self.players[target],
                        Hit {
                            damage: SHIELD_DAMAGE,
                            hitstun: 18,
                            blockstun: 10,
                            knockback: Fx::ratio(7, 1),
                            dir,
                            blocked: victim.action.guarding(),
                            parried: false,
                        },
                    );
                    // Contact drops it where it struck.
                    self.players[owner].mechanic = Mechanic::Shield(Shield::Planted { pos });
                }
            }
        }

        separate_bodies(&mut self.players);

        // Knockout check last, so the killing blow is fully applied first.
        if matches!(self.phase, Phase::Fighting) {
            let down = [self.players[0].health <= 0, self.players[1].health <= 0];
            if down[0] || down[1] {
                let winner = match down {
                    [true, true] => u8::MAX,
                    [true, false] => 1,
                    _ => 0,
                };
                if winner != u8::MAX {
                    self.players[winner as usize].rounds_won += 1;
                }
                self.phase = Phase::RoundOver {
                    winner,
                    left: t::ROUND_OVER_FRAMES,
                };
            }
        }
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
            h.write_u32(p.grounded as u32);
            h.write_u32(p.hit_used as u32);
            h.write_u32(p.crouching as u32);
            h.write_u32(p.action.tag());
            h.write_u32(p.action.frames_left() as u32);
            let kind = match p.action {
                Action::Startup { kind, .. }
                | Action::Active { kind, .. }
                | Action::Recovery { kind, .. } => kind,
                _ => 0,
            };
            h.write_u32(kind as u32);
            h.write_u32(p.rounds_won as u32);
            h.write_u32(p.class as u32);
            hash_mechanic(&mut h, &p.mechanic);
        }
        match self.phase {
            Phase::Fighting => h.write_u32(0),
            Phase::RoundOver { winner, left } => {
                h.write_u32(1);
                h.write_u32(winner as u32);
                h.write_u32(left as u32);
            }
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
    if attacker.hit_used || defender.action.invulnerable() {
        return None;
    }
    let m = moves::get(attacker.class, kind);

    // Overheads miss a crouching defender. Expressed as a property of the move
    // rather than as hitbox geometry, because that is what players read and
    // what a frame table can state.
    if defender.crouching && !m.hits_crouching {
        return None;
    }

    // The Bellator's form multiplies reach and damage rather than each form
    // having its own table -- three numbers instead of three move lists.
    let (reach_mul, damage_mul) = match attacker.mechanic {
        Mechanic::Forms { form, .. } => {
            let (r, d, _) = form.modifiers();
            (r, d)
        }
        _ => (Fx::ONE, Fx::ONE),
    };

    let centre = attacker
        .pos
        .add(attacker.facing.scale(m.reach.mul(reach_mul)));
    let delta = defender.pos.sub(centre);
    if delta.flat_len().raw() > m.radius.add(t::BODY_RADIUS).raw() {
        return None;
    }

    // Was the defender facing the attack? Guard covers an arc, not a bubble.
    let to_attacker = attacker.pos.sub(defender.pos).normalized();
    let facing_it = defender.facing.dot(to_attacker).raw() >= t::GUARD_ARC_COS.raw();
    // A grapple goes through guard entirely. That is what stops blocking from
    // being a solved strategy -- see defense.md.
    let guarding = !m.unblockable && defender.action.guarding() && facing_it;
    let parried = !m.unblockable
        && matches!(defender.action, Action::Guard { held } if held < t::PARRY_WINDOW)
        && facing_it;

    Some(Hit {
        damage: Fx::from_int(m.damage).mul(damage_mul).to_int(),
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
            t::GUARD_TURN_RATE
        } else {
            t::TURN_RATE
        };
        p.facing = p.facing.add(target.sub(p.facing).scale(rate)).normalized();
    }

    step_mechanic(p);

    let want_guard = input.has(Input::RIGHT) && p.shield().is_some_and(|sh| sh.in_hand());
    let (ax, az) = input.move_axis();
    // Crouch is a stance, not an action: it holds while the key is down and
    // only while you are otherwise free to move.
    p.crouching = input.has(Input::CROUCH) && p.grounded && p.action.actionable();

    p.action = match p.action {
        Action::Dodge { left } if left > 0 => Action::Dodge { left: left - 1 },
        Action::Dodge { .. } => Action::Free,
        Action::Startup { kind, left } if left > 0 => Action::Startup {
            kind,
            left: left - 1,
        },
        Action::Startup { kind, .. } => Action::Active {
            kind,
            left: moves::get(p.class, kind).active,
        },
        Action::Active { kind, left } if left > 0 => Action::Active {
            kind,
            left: left - 1,
        },
        Action::Active { kind, .. } => Action::Recovery {
            kind,
            left: moves::get(p.class, kind).recovery,
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
            // Space plus a direction dodges; space alone jumps. See controls.md.
            if input.has(Input::SPACE) && (ax != 0 || az != 0) && p.grounded {
                let dir = V3::new(Fx::from_int(ax), Fx::ZERO, Fx::from_int(az)).normalized();
                p.vel.x = dir.x.mul(t::DODGE_SPEED);
                p.vel.z = dir.z.mul(t::DODGE_SPEED);
                Action::Dodge {
                    left: t::DODGE_FRAMES,
                }
            } else if input.has(Input::MIDDLE)
                && input.has(Input::SHIFT)
                && p.shield().is_some_and(|sh| sh.in_hand())
            {
                p.hit_used = false;
                Action::Startup {
                    kind: SLOT_SPECIAL,
                    left: moves::get(p.class, 2).startup,
                }
            } else if input.has(Input::MIDDLE) {
                mechanic_action(p);
                Action::Free
            } else if input.has(Input::LEFT) && p.mechanic_ready(SLOT_POKE) {
                let kind = if input.has(Input::SHIFT) {
                    SLOT_COMMITTED
                } else {
                    SLOT_POKE
                };
                p.hit_used = false;
                steer_meter(p, input, kind);
                Action::Startup {
                    kind,
                    left: moves::get(p.class, kind).startup,
                }
            } else if want_guard {
                Action::Guard { held: 0 }
            } else {
                Action::Free
            }
        }
    };

    // Horizontal movement. Free at full speed, guarding at a crawl, otherwise
    // only carried momentum from a dodge or knockback.
    if matches!(p.action, Action::Dodge { .. }) {
        p.vel.x = p.vel.x.mul(Fx::ratio(93, 100));
        p.vel.z = p.vel.z.mul(Fx::ratio(93, 100));
    } else if p.action.actionable() && (ax != 0 || az != 0) {
        let speed = if p.crouching {
            t::CROUCH_MOVE_SPEED
        } else {
            t::MOVE_SPEED
        };
        let dir = V3::new(Fx::from_int(ax), Fx::ZERO, Fx::from_int(az)).normalized();
        p.vel.x = dir.x.mul(speed);
        p.vel.z = dir.z.mul(speed);
    } else if p.action.guarding() && (ax != 0 || az != 0) {
        let dir = V3::new(Fx::from_int(ax), Fx::ZERO, Fx::from_int(az)).normalized();
        p.vel.x = dir.x.mul(t::GUARD_MOVE_SPEED);
        p.vel.z = dir.z.mul(t::GUARD_MOVE_SPEED);
    } else if p.action.stunned() {
        // Knockback decays rather than stopping dead.
        p.vel.x = p.vel.x.mul(Fx::ratio(86, 100));
        p.vel.z = p.vel.z.mul(Fx::ratio(86, 100));
    } else {
        p.vel.x = Fx::ZERO;
        p.vel.z = Fx::ZERO;
    }

    if input.has(Input::SPACE) && ax == 0 && az == 0 && p.grounded && p.action.actionable() {
        p.vel.y = t::JUMP_SPEED;
        p.grounded = false;
    }

    if !p.grounded {
        p.vel.y = p.vel.y.add(t::GRAVITY.mul(DT));
    }

    p.pos = p.pos.add(p.vel.scale(DT));

    let r = arena::resolve(p.pos, p.vel, p.grounded);
    p.pos = r.pos;
    p.vel = r.vel;
    p.grounded = r.grounded;
}

/// The middle-click mechanic action, per class.
///
/// One button means something different on every class, which is where the
/// identity lives -- see `controls.md`. Everything else about the control
/// scheme is shared.
fn mechanic_action(p: &mut Player) {
    match p.mechanic {
        // Throw commits you: faster, exposed, and unable to block until it is
        // back. Recall damages along the return path; reactivating mid-flight
        // leaps you to it, which is the Bulwark's approach tool.
        Mechanic::Shield(shield) => {
            p.mechanic = Mechanic::Shield(match shield {
                Shield::Held => Shield::Flying {
                    pos: p.pos.add(V3::new(Fx::ZERO, Fx::ONE, Fx::ZERO)),
                    vel: p.facing.scale(SHIELD_SPEED),
                    outbound: true,
                    travelled: Fx::ZERO,
                },
                Shield::Planted { pos } => {
                    let to_owner = p.pos.add(V3::new(Fx::ZERO, Fx::ONE, Fx::ZERO)).sub(pos);
                    Shield::Flying {
                        pos,
                        vel: to_owner.normalized().scale(SHIELD_SPEED),
                        outbound: false,
                        travelled: Fx::ZERO,
                    }
                }
                Shield::Flying {
                    pos,
                    vel,
                    outbound,
                    travelled,
                } => {
                    let to_shield = pos.sub(p.pos);
                    if to_shield.flat_len().raw() > Fx::ONE.raw() {
                        let dir = V3::new(to_shield.x, Fx::ZERO, to_shield.z).normalized();
                        p.vel.x = dir.x.mul(LEAP_SPEED);
                        p.vel.z = dir.z.mul(LEAP_SPEED);
                        p.vel.y = Fx::ratio(6, 1);
                        p.grounded = false;
                    }
                    Shield::Flying {
                        pos,
                        vel,
                        outbound,
                        travelled,
                    }
                }
            });
        }

        // Cycle the weapon form. Every move's reach, damage and recovery are
        // multiplied by it, so this is three kits from one table.
        Mechanic::Forms { form, rush_charged } => {
            let next = match form {
                Form::Hammer => Form::Sword,
                Form::Sword => Form::Spear,
                Form::Spear => Form::Hammer,
            };
            p.mechanic = Mechanic::Forms {
                form: next,
                rush_charged,
            };
        }

        // Place the shadow ahead, or reclaim it. Mobility and setup are the
        // same action, which is what keeps the Reaver from being denied its
        // movement.
        Mechanic::Shadow { at } => {
            p.mechanic = Mechanic::Shadow {
                at: match at {
                    None => Some(p.pos.add(p.facing.scale(Fx::from_int(3)))),
                    Some(_) => None,
                },
            };
        }

        // Spawn a structure ahead. A fourth collapses the oldest, so the cap
        // is the resource.
        Mechanic::Structures(mut slots) => {
            let spot = p.pos.add(p.facing.scale(Fx::ratio(5, 2)));
            if let Some(free) = slots.iter_mut().find(|s| s.is_none()) {
                *free = Some(spot);
            } else {
                slots.rotate_left(1);
                slots[class::MAX_STRUCTURES - 1] = Some(spot);
            }
            p.mechanic = Mechanic::Structures(slots);
        }

        // Health is the resource; there is no separate button.
        Mechanic::Blood => {}

        // The meter is steered by which button attacks, not by this one.
        Mechanic::Meter { .. } => {}
    }
}

/// Per-tick mechanic upkeep: shields in flight, shadows on a leash, the meter
/// burning at depth.
fn step_mechanic(p: &mut Player) {
    match p.mechanic {
        Mechanic::Shield(Shield::Flying {
            pos,
            vel,
            outbound,
            travelled,
        }) => {
            let step = vel.scale(DT);
            let next = pos.add(step);
            let gone = travelled.add(step.flat_len());
            p.mechanic = Mechanic::Shield(if outbound {
                if gone.raw() >= SHIELD_RANGE.raw() {
                    Shield::Planted { pos: next }
                } else {
                    Shield::Flying {
                        pos: next,
                        vel,
                        outbound,
                        travelled: gone,
                    }
                }
            } else {
                let hand = p.pos.add(V3::new(Fx::ZERO, Fx::ONE, Fx::ZERO));
                if next.sub(hand).flat_len().raw() < Fx::ONE.raw() {
                    Shield::Held
                } else {
                    // Home in, so the return does not miss a moving owner.
                    let dir = hand.sub(next).normalized().scale(SHIELD_SPEED);
                    Shield::Flying {
                        pos: next,
                        vel: dir,
                        outbound,
                        travelled: gone,
                    }
                }
            });
        }

        // Leaving the leash snaps the shadow back.
        Mechanic::Shadow { at: Some(spot) } => {
            if spot.sub(p.pos).flat_len().raw() > SHADOW_LEASH.raw() {
                p.mechanic = Mechanic::Shadow { at: None };
            }
        }

        // Past the deep threshold the forces burn you. Relief comes from
        // coming back inside the line, not from a reward -- see dual-mage.md.
        Mechanic::Meter { value } => {
            let depth = value.abs();
            if depth > METER_DEEP {
                let over = depth - METER_DEEP;
                let span = (METER_MAX - METER_DEEP).max(1);
                let burn = (BURN_PER_TICK_AT_MAX * over) / span;
                p.health = (p.health - burn.max(1)).max(1);
            }
        }

        _ => {}
    }
}

/// Attacking steers the Dual mage's meter: left darker, right lighter, and
/// stronger moves push harder. Nothing else moves it, so every step is a
/// consequence of a decision the player made.
fn steer_meter(p: &mut Player, input: Input, kind: u8) {
    let Mechanic::Meter { value } = p.mechanic else {
        return;
    };
    let push = 4 + (moves::get(p.class, kind).damage / 40);
    let delta = if input.has(Input::LEFT) {
        -push
    } else if input.has(Input::RIGHT) {
        push
    } else {
        0
    };
    p.mechanic = Mechanic::Meter {
        value: (value + delta).clamp(-METER_MAX, METER_MAX),
    };
}

fn hash_mechanic(h: &mut Fnv, m: &Mechanic) {
    match m {
        Mechanic::Shield(Shield::Held) => h.write_u32(0),
        Mechanic::Shield(Shield::Planted { pos }) => {
            h.write_u32(1);
            hash_v3(h, pos);
        }
        Mechanic::Shield(Shield::Flying {
            pos,
            vel,
            outbound,
            travelled,
        }) => {
            h.write_u32(2);
            hash_v3(h, pos);
            hash_v3(h, vel);
            h.write_u32(*outbound as u32);
            h.write_i32(travelled.raw());
        }
        Mechanic::Forms { form, rush_charged } => {
            h.write_u32(3);
            h.write_u32(*form as u32);
            h.write_u32(*rush_charged as u32);
        }
        Mechanic::Shadow { at } => {
            h.write_u32(4);
            match at {
                Some(pos) => hash_v3(h, pos),
                None => h.write_u32(0),
            }
        }
        Mechanic::Structures(slots) => {
            h.write_u32(5);
            for slot in slots {
                match slot {
                    Some(pos) => hash_v3(h, pos),
                    None => h.write_u32(0),
                }
            }
        }
        Mechanic::Blood => h.write_u32(6),
        Mechanic::Meter { value } => {
            h.write_u32(7);
            h.write_i32(*value);
        }
    }
}

fn hash_v3(h: &mut Fnv, v: &V3) {
    h.write_i32(v.x.raw());
    h.write_i32(v.y.raw());
    h.write_i32(v.z.raw());
}

/// Let a body come to rest without accepting input. Used during the pause
/// between rounds.
fn settle(p: &mut Player) {
    p.vel.x = p.vel.x.mul(Fx::ratio(88, 100));
    p.vel.z = p.vel.z.mul(Fx::ratio(88, 100));
    if !p.grounded {
        p.vel.y = p.vel.y.add(t::GRAVITY.mul(DT));
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
    let min = t::BODY_RADIUS.add(t::BODY_RADIUS);
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
pub fn move_frames(class: Class, kind: u8) -> (u16, u16, u16) {
    moves::frames(class, kind)
}

/// Parry window length, exposed for debug overlays.
pub const fn parry_window() -> u16 {
    t::PARRY_WINDOW
}
