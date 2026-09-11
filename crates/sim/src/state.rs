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
use crate::effects::{Effect, EffectKind, MAX_EFFECTS};
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
/// `held_by` when nobody is holding you.
pub const NOBODY: u8 = u8::MAX;

pub const SLOT_POKE: u8 = 0;
pub const SLOT_COMMITTED: u8 = 1;
pub const SLOT_SPECIAL: u8 = 2;
pub use crate::tuning::max_health;

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
    /// `held` counts up. The first `t::parry_window()` frames parry.
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
    /// Caught by a grab. You cannot act, and you are dragged to whoever has
    /// you, which is what makes a grab different from a long stun: it moves
    /// you, so the grabber decides where the next exchange happens.
    Held {
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

    /// Which move is running, across all three of its phases.
    pub const fn attack_kind(self) -> Option<u8> {
        match self {
            Action::Startup { kind, .. }
            | Action::Active { kind, .. }
            | Action::Recovery { kind, .. } => Some(kind),
            _ => None,
        }
    }

    /// Invulnerable frames of a dodge. Nothing else grants invulnerability.
    pub fn invulnerable(self) -> bool {
        matches!(self, Action::Dodge { left } if left + t::dodge_iframes() > t::dodge_frames())
    }

    /// Blocking costs a vulnerable window, so it is never a safe default.
    /// See `defense.md`.
    pub const fn stunned(self) -> bool {
        matches!(
            self,
            Action::BlockStun { .. }
                | Action::HitStun { .. }
                | Action::Stagger { .. }
                | Action::Held { .. }
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
            Action::Held { .. } => 9,
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
            | Action::Held { left }
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
    /// Frames of jump sustain left. Set on takeoff, spent while the button is
    /// held and you are rising, and zeroed the moment it is released -- so a
    /// re-press cannot extend a jump you already let go of.
    pub jump_hold: u16,
    /// Frames of suspended fall from an aerial attack. See `Move::air_stall`.
    pub air_stall: u16,
    /// An airdodge has been spent this airtime. Reset on landing.
    ///
    /// Without it, airdodging repeatedly is free flight: each one is a fresh
    /// burst of horizontal speed with invulnerability attached.
    pub air_dodged: bool,
    /// True once the current active window has connected, so a move hits once.
    pub hit_used: bool,
    pub class: Class,
    pub mechanic: Mechanic,
    pub rounds_won: u8,
    pub crouching: bool,
    /// Frames of slow left. A drain field sets it and it counts down, so the
    /// slow has a tail and does not flicker on the field's boundary.
    pub slowed: u16,
    /// Who is holding this fighter, or `u8::MAX`. A grab has to know its owner
    /// so the victim can be kept at arm's length rather than merely stunned.
    pub held_by: u8,
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
            arena::BODY_HEIGHT.mul(t::crouch_height_scale())
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
            jump_hold: 0,
            air_stall: 0,
            air_dodged: false,
            hit_used: false,
            class: Class::Bulwark,
            mechanic: Mechanic::Shield(Shield::Held),
            rounds_won: 0,
            crouching: false,
            slowed: 0,
            held_by: NOBODY,
        }
    }
}

/// The complete rollback snapshot.
#[derive(Clone, PartialEq, Debug)]
pub struct World {
    pub frame: u32,
    pub players: [Player; MAX_PLAYERS],
    pub phase: Phase,
    /// Things moves have left behind. Fixed size: see `effects`.
    pub effects: [Option<Effect>; MAX_EFFECTS],
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
            effects: [None; MAX_EFFECTS],
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

        for (p, input) in self.players.iter_mut().zip(inputs) {
            step_player(p, input);
        }

        // What a move does *as it comes out*, on its first active frame: the
        // leap of a leaping move, and whatever it leaves standing in the world.
        for i in 0..MAX_PLAYERS {
            let p = self.players[i];
            let Action::Active { kind, left } = p.action else {
                continue;
            };
            let m = moves::get(p.class, kind);
            if left != m.active {
                continue;
            }
            if m.self_lift.raw() > 0 {
                self.players[i].vel.y = m.self_lift;
                self.players[i].grounded = false;
            }
            if let Some(kind) = EffectKind::from_code(m.effect) {
                let spot = p.pos.add(p.facing.scale(m.reach));
                spawn_effect(
                    &mut self.effects,
                    kind,
                    i as u8,
                    V3::new(spot.x, Fx::ZERO, spot.z),
                );
            }
        }

        // Hit resolution after both have stepped, so neither ordering wins.
        let snapshot = self.players;
        for attacker in 0..MAX_PLAYERS {
            let defender = 1 - attacker;
            if let Some(hit) = resolve_hit(&snapshot[attacker], &snapshot[defender], attacker as u8)
            {
                apply_hit(&mut self.players[defender], hit);
                self.players[attacker].hit_used = true;
                if hit.parried {
                    self.players[attacker].action = Action::Stagger {
                        left: t::parry_stagger(),
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
                let hit_range = SHIELD_RADIUS.add(t::body_radius());
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
                            launch: Fx::ZERO,
                            grabs: 0,
                            by: owner as u8,
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

        step_effects(&mut self.effects, &mut self.players);
        separate_bodies(&mut self.players);
        drag_the_held(&mut self.players);

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
                    left: t::round_over_frames(),
                };
            }
        }
    }

    /// Cheap and portable. Used for desync detection: peers exchange checksums
    /// periodically and a mismatch means the simulations diverged.
    /// Snapshot hash, including the tuning.
    ///
    /// The Oven's values are rules rather than state, so rollback never saves or
    /// restores them -- but two peers running different rules would diverge
    /// silently and look exactly like a netcode bug. Folding the tuning hash in
    /// turns that into a desync on the first frame, which is a error message
    /// rather than a mystery.
    pub fn checksum(&self) -> u64 {
        let mut h = Fnv::new();
        h.write_u64(crate::oven::hash());
        h.write_u32(self.frame);
        for e in &self.effects {
            match e {
                Some(e) => {
                    h.write_u32(e.kind as u32 + 1);
                    h.write_u32(e.owner as u32);
                    h.write_u32(e.age as u32);
                    h.write_u32(e.life as u32);
                    hash_v3(&mut h, &e.pos);
                }
                None => h.write_u32(0),
            }
        }
        for p in &self.players {
            for v in [p.pos, p.vel, p.facing] {
                h.write_i32(v.x.raw());
                h.write_i32(v.y.raw());
                h.write_i32(v.z.raw());
            }
            h.write_i32(p.health);
            h.write_u32(p.grounded as u32);
            h.write_u32(p.air_dodged as u32);
            h.write_u32(p.slowed as u32);
            h.write_u32(p.held_by as u32);
            h.write_u32(p.jump_hold as u32);
            h.write_u32(p.air_stall as u32);
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
    /// Upward speed handed to the victim. This is what takes someone off the
    /// ground with an uppercut instead of shoving them along it.
    launch: Fx,
    /// Frames the victim is held at the attacker's arm's length. Zero is a
    /// normal hit.
    grabs: u16,
    /// Who threw it, so a grab knows whose arm to hang from.
    by: u8,
    dir: V3,
    blocked: bool,
    parried: bool,
}

/// The attack volume a fighter currently has out.
///
/// Exists so the debug overlay draws *the thing the hit test uses* rather than
/// its own reconstruction of it. An overlay that can drift from the rule it
/// illustrates is worse than no overlay: it is confidently wrong at exactly the
/// moment you are trying to work out why something did not connect.
#[derive(Clone, Copy, Debug)]
pub struct Hitbox {
    /// Flat centre, at the attacker's own height.
    pub centre: V3,
    /// The attack's radius. A defender is hit when their body circle overlaps
    /// this one, so the test threshold is this plus `BODY_RADIUS`.
    pub radius: Fx,
    /// False means an overhead: it passes over a crouching defender.
    pub hits_crouching: bool,
    pub unblockable: bool,
    /// The move has already connected this swing and cannot connect again.
    /// Still drawn, because it is still visibly out.
    pub spent: bool,
}

/// The attack volume out this frame, if any. `None` outside active frames.
pub fn hitbox(p: &Player) -> Option<Hitbox> {
    let Action::Active { kind, .. } = p.action else {
        return None;
    };
    let m = moves::get(p.class, kind);
    // The Bellator's form multiplies reach rather than each form having its own
    // table. Applying it here, once, is why the overlay cannot disagree with
    // the hit test about where a spear reaches.
    let reach_mul = match p.mechanic {
        Mechanic::Forms { form, .. } => form.modifiers().0,
        _ => Fx::ONE,
    };
    Some(Hitbox {
        centre: p.pos.add(p.facing.scale(m.reach.mul(reach_mul))),
        radius: m.radius,
        hits_crouching: m.hits_crouching,
        unblockable: m.unblockable,
        spent: p.hit_used,
    })
}

fn resolve_hit(attacker: &Player, defender: &Player, by: u8) -> Option<Hit> {
    let Action::Active { kind, .. } = attacker.action else {
        return None;
    };
    let box_out = hitbox(attacker)?;
    if box_out.spent || defender.action.invulnerable() {
        return None;
    }
    let m = moves::get(attacker.class, kind);

    // Overheads miss a crouching defender. Expressed as a property of the move
    // rather than as hitbox geometry, because that is what players read and
    // what a frame table can state.
    if defender.crouching && !box_out.hits_crouching {
        return None;
    }

    let damage_mul = match attacker.mechanic {
        Mechanic::Forms { form, .. } => form.modifiers().1,
        _ => Fx::ONE,
    };

    let delta = defender.pos.sub(box_out.centre);
    if delta.flat_len().raw() > box_out.radius.add(t::body_radius()).raw() {
        return None;
    }

    // Was the defender facing the attack? Guard covers an arc, not a bubble.
    let to_attacker = attacker.pos.sub(defender.pos).normalized();
    let facing_it = defender.facing.dot(to_attacker).raw() >= t::guard_arc_cos().raw();
    // A grapple goes through guard entirely. That is what stops blocking from
    // being a solved strategy -- see defense.md.
    let guarding = !m.unblockable && defender.action.guarding() && facing_it;
    let parried = !m.unblockable
        && matches!(defender.action, Action::Guard { held } if held < t::parry_window())
        && facing_it;

    Some(Hit {
        damage: Fx::from_int(m.damage).mul(damage_mul).to_int(),
        hitstun: m.hitstun,
        blockstun: m.blockstun,
        knockback: m.knockback,
        launch: m.launch,
        grabs: m.grabs,
        by,
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
        defender.vel.x = hit.dir.x.mul(hit.knockback);
        defender.vel.z = hit.dir.z.mul(hit.knockback);
        if hit.grabs > 0 {
            // A grab is not knockback. The victim is pinned to the grabber and
            // goes wherever they go, which is what makes a grab a commitment
            // for *both* of them rather than a shove with a longer stun.
            defender.action = Action::Held { left: hit.grabs };
            defender.held_by = hit.by;
            defender.vel.x = Fx::ZERO;
            defender.vel.z = Fx::ZERO;
        } else {
            defender.action = Action::HitStun { left: hit.hitstun };
        }
        if hit.launch.raw() > 0 {
            // Taken off the ground. Launch *sets* vertical speed rather than
            // adding to it, so being hit on the way down does not cancel out.
            defender.vel.y = hit.launch;
            defender.grounded = false;
        }
    }
}

/// Turn a stick reading into a world direction, given where the player looks.
///
/// `W` is away from the camera, `D` is to its right. This is the whole of
/// "camera-relative movement", and it is why aim has to be simulation input:
/// the same button means a different direction depending on it.
pub fn move_dir(aim: Fx, ax: i32, az: i32) -> V3 {
    if ax == 0 && az == 0 {
        return V3::ZERO;
    }
    let forward = V3::from_turns(aim);
    let right = V3::from_turns(aim.add(QUARTER_TURN));
    forward
        .scale(Fx::from_int(az))
        .add(right.scale(Fx::from_int(ax)))
        .normalized()
}

/// A quarter turn in `Fx`, matching `Input::QUARTER_TURN`.
const QUARTER_TURN: Fx = Fx::from_raw(1 << 14);

fn step_player(p: &mut Player, input: Input) {
    // Facing comes from the mouse. Where you look is where you are pointed, and
    // where you are pointed is where your attacks go.
    //
    // Two exceptions, and both are load-bearing:
    //
    // Once a move has started, facing is **locked**. Otherwise the mouse would
    // drag a live hitbox around during its active frames, and a whiff could be
    // rescued by turning after the fact -- which would take whiff punishment,
    // most of the game, out behind the shed. Commitment is spatial here; you
    // commit to a direction when you commit to the move.
    //
    // While guarding, facing turns at a limited rate. Guard covers an arc, not
    // a bubble (see defense.md), and an arc you can flip instantly is a bubble
    // with extra steps. The camera still snaps wherever the mouse goes -- it is
    // the character who cannot reorient that fast.
    let look = V3::from_turns(input.aim_turns());
    if p.action.actionable() || p.action.stunned() {
        p.facing = look;
    } else if p.action.guarding() {
        p.facing = p
            .facing
            .add(look.sub(p.facing).scale(t::guard_turn_rate()))
            .normalized();
    }

    let mob = p.class.mobility();
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
        Action::Held { left } if left > 0 => Action::Held { left: left - 1 },
        Action::Held { .. } => {
            p.held_by = NOBODY;
            Action::Free
        }
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
            // Clicks are checked before the dodge, which is what disambiguates
            // shift. Shift with a click is the stronger version of that attack;
            // shift with only a direction is a dodge. See controls.md.
            if input.has(Input::SPECIAL) && p.mechanic_ready(SLOT_SPECIAL) {
                p.hit_used = false;
                arm_aerial(p, SLOT_SPECIAL, input);
                Action::Startup {
                    kind: SLOT_SPECIAL,
                    left: moves::get(p.class, 2).startup,
                }
            } else if input.has(Input::MECHANIC) {
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
                arm_aerial(p, kind, input);
                Action::Startup {
                    kind,
                    left: moves::get(p.class, kind).startup,
                }
            } else if input.has(Input::SHIFT) && !input.any_click() && (ax != 0 || az != 0) {
                // Shift plus a direction dodges. It used to be space plus a
                // direction, which meant that pressing the jump button while
                // moving -- which is most of the time -- did not jump. Space is
                // now only ever a vertical takeoff.
                let dir = move_dir(input.aim_turns(), ax, az);
                if p.grounded {
                    p.vel.x = dir.x.mul(t::dodge_speed());
                    p.vel.z = dir.z.mul(t::dodge_speed());
                    Action::Dodge {
                        left: t::dodge_frames(),
                    }
                } else if !p.air_dodged {
                    // An airdodge, once per airtime. It commits you to a
                    // direction in the air, where you otherwise have almost no
                    // say, which is why it can only be spent once: a second one
                    // would turn a jump into flight.
                    p.air_dodged = true;
                    p.vel.x = dir.x.mul(t::air_dodge_speed());
                    p.vel.z = dir.z.mul(t::air_dodge_speed());
                    // Vertical speed is wiped rather than added to, so an
                    // airdodge is a sideways commitment and never a second jump.
                    p.vel.y = Fx::ZERO;
                    Action::Dodge {
                        left: t::air_dodge_frames(),
                    }
                } else {
                    Action::Free
                }
            } else if want_guard {
                Action::Guard { held: 0 }
            } else {
                Action::Free
            }
        }
    };

    // Horizontal movement.
    //
    // How much a move hinders you is proportional to how much it commits you.
    // The heavy moves root you outright -- that is what commitment means, and
    // it is the whole basis of spatial play here. A fast poke does not: it is
    // the neutral tool, thrown constantly, and stopping dead every time made
    // neutral sticky and read as the game taking the controls away.
    let attack_speed = p
        .action
        .attack_kind()
        .map(|kind| moves::get(p.class, kind).mobility)
        .filter(|m| *m > 0)
        .map(|m| t::move_speed().mul(Fx::ratio(m as i32, 100)));
    let steering = ax != 0 || az != 0;

    if matches!(p.action, Action::Dodge { .. }) {
        p.vel.x = p.vel.x.mul(Fx::ratio(93, 100));
        p.vel.z = p.vel.z.mul(Fx::ratio(93, 100));
    } else if p.action.stunned() {
        // Knockback decays rather than stopping dead, in the air as on the
        // ground. Checked before the airborne branch so a hit connecting
        // mid-jump is not immediately steered out of.
        p.vel.x = p.vel.x.mul(Fx::ratio(86, 100));
        p.vel.z = p.vel.z.mul(Fx::ratio(86, 100));
    } else if !p.grounded {
        // In the air, input *accelerates* rather than assigns. Momentum is
        // conserved when you let go, which is the whole difference between a
        // jump being a commitment and a jump being a hover.
        if steering {
            air_accelerate(p, move_dir(input.aim_turns(), ax, az), mob.air_speed);
        }
    } else if p.action.actionable() && steering {
        let speed = if p.crouching {
            t::crouch_move_speed()
        } else {
            t::move_speed()
        };
        let dir = move_dir(input.aim_turns(), ax, az);
        p.vel.x = dir.x.mul(dragged(p, speed));
        p.vel.z = dir.z.mul(dragged(p, speed));
    } else if p.action.guarding() && steering {
        let dir = move_dir(input.aim_turns(), ax, az);
        p.vel.x = dir.x.mul(dragged(p, t::guard_move_speed()));
        p.vel.z = dir.z.mul(dragged(p, t::guard_move_speed()));
    } else if let (Some(speed), true) = (attack_speed, steering) {
        let dir = move_dir(input.aim_turns(), ax, az);
        p.vel.x = dir.x.mul(dragged(p, speed));
        p.vel.z = dir.z.mul(dragged(p, speed));
    } else if p.action.attack_kind().is_some() {
        // Rooted, or steering nothing. Bleed the speed off over a few frames
        // rather than snapping to a halt: the snap was the jarring part, not
        // the rooting.
        p.vel.x = p.vel.x.mul(t::attack_root_decay());
        p.vel.z = p.vel.z.mul(t::attack_root_decay());
    } else {
        p.vel.x = Fx::ZERO;
        p.vel.z = Fx::ZERO;
    }

    // Space is a vertical takeoff, whatever your feet are doing. Holding a
    // direction while jumping carries your momentum up with you; it does not
    // turn the jump into something else.
    if input.has(Input::SPACE) && p.grounded && p.action.actionable() {
        p.vel.y = t::jump_speed().mul(mob.jump);
        p.grounded = false;
        p.jump_hold = t::jump_hold_frames();
    }

    if !p.grounded {
        if p.air_stall > 0 {
            // An aerial hangs you for a few frames: gravity is held off, and
            // whatever vertical speed you had **bleeds away** rather than being
            // deleted.
            //
            // Deleting it read as the game snatching the jump out from under
            // you mid-rise. The extra control over jump height that attacking
            // gives is worth keeping -- it just has to arrive as a slowing
            // rather than a stop, which is also what makes the hang read as
            // float rather than a pause. Gravity staying off through the window
            // is what keeps it punchy: you hang, you do not sag.
            p.air_stall -= 1;
            p.vel.y = p.vel.y.mul(t::air_stall_damp());
        } else {
            // Holding the jump button sustains the rise. Releasing ends it for
            // good -- `jump_hold` goes to zero rather than pausing, so tapping
            // again cannot resurrect a jump you already cut short.
            let sustaining = input.has(Input::SPACE) && p.jump_hold > 0 && p.vel.y.raw() > 0;
            // Letting go while still rising cuts what is left of the climb,
            // once. `jump_hold` drops to zero immediately after, so a second
            // press can neither cut again nor resurrect the jump.
            if !sustaining && p.jump_hold > 0 && p.vel.y.raw() > 0 {
                p.vel.y = p.vel.y.mul(t::jump_release_cut());
            }
            if sustaining {
                p.jump_hold -= 1;
            } else {
                p.jump_hold = 0;
            }
            let mut gravity = t::gravity().mul(mob.gravity);
            if sustaining {
                gravity = gravity.mul(t::jump_hold_gravity());
            }
            p.vel.y = p.vel.y.add(gravity.mul(DT));
            let floor = t::fall_cap().mul(mob.fall_cap);
            if p.vel.y.raw() < floor.raw() {
                p.vel.y = floor;
            }
        }
    }

    p.pos = p.pos.add(p.vel.scale(DT));

    let r = arena::resolve(p.pos, p.vel, p.grounded);
    p.pos = r.pos;
    p.vel = r.vel;
    p.grounded = r.grounded;
    if p.grounded {
        p.air_dodged = false;
        p.jump_hold = 0;
        p.air_stall = 0;
    }
}

/// Start an aerial's hang, and its shove, if this one is thrown in the air.
///
/// The shove is the basic attack's alone. A poke is the move you throw
/// constantly, so a small push in the direction you are holding is what makes
/// attacking *part of* air movement rather than a pause in it: the hang
/// supplies the float, the shove supplies the punch. The committed moves
/// deliberately get none -- they are already a commitment, and one that also
/// repositioned you would be strictly better than a poke.
fn arm_aerial(p: &mut Player, kind: u8, input: Input) {
    if p.grounded {
        return;
    }
    p.air_stall = moves::get(p.class, kind).air_stall;

    let (ax, az) = input.move_axis();
    if kind != SLOT_POKE || (ax == 0 && az == 0) {
        return;
    }
    let dir = move_dir(input.aim_turns(), ax, az);
    let boost = t::air_attack_boost();
    p.vel.x = p.vel.x.add(dir.x.mul(boost));
    p.vel.z = p.vel.z.add(dir.z.mul(boost));
    clamp_air_speed(p);
}

/// Hold horizontal air speed under the ceiling. See `tuning::air_speed_cap`.
fn clamp_air_speed(p: &mut Player) {
    let cap = t::move_speed().mul(t::air_speed_cap());
    let speed = V3::new(p.vel.x, Fx::ZERO, p.vel.z).flat_len();
    if speed.raw() > cap.raw() {
        let scale = cap.div(speed);
        p.vel.x = p.vel.x.mul(scale);
        p.vel.z = p.vel.z.mul(scale);
    }
}

/// Quake-style air acceleration, which is where air control gets its skill
/// ceiling.
///
/// The whole trick is one line: `current` is the velocity **projected onto the
/// direction you asked for**. Point where you are already going and the
/// projection is large, so there is nothing left to add and holding forward does
/// almost nothing. Point perpendicular to your motion and the projection is near
/// zero, so you get the full budget -- which *turns* the velocity vector without
/// spending it.
///
/// That is why strafing is expressive and holding forward is not: the game is
/// not rewarding a faster input, it is rewarding an input aimed at the component
/// of your motion you have not already used up.
fn air_accelerate(p: &mut Player, wish: V3, wish_speed: Fx) {
    let current = p.vel.x.mul(wish.x).add(p.vel.z.mul(wish.z));
    let head_room = wish_speed.sub(current);
    if head_room.raw() <= 0 {
        return;
    }
    let mut step = t::air_accel().mul(wish_speed).mul(DT);
    if step.raw() > head_room.raw() {
        step = head_room;
    }
    p.vel.x = p.vel.x.add(wish.x.mul(step));
    p.vel.z = p.vel.z.add(wish.z.mul(step));

    // A ceiling Source does not have. See `tuning::air_speed_cap`.
    clamp_air_speed(p);
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
        p.vel.y = p.vel.y.add(t::gravity().mul(DT));
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
    let min = t::body_radius().add(t::body_radius());
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
    fn write_u64(&mut self, v: u64) {
        self.write_u32(v as u32);
        self.write_u32((v >> 32) as u32);
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
pub fn parry_window() -> u16 {
    t::parry_window()
}

/// Age every effect, apply what it does, and drop the expired.
///
/// Effects act *after* both fighters have stepped, so standing in a field for a
/// frame costs you whether you walked in or were knocked in. The alternative --
/// checking before movement -- would let someone walk through a fire pillar
/// untouched on the frame they entered it.
fn step_effects(effects: &mut [Option<Effect>; MAX_EFFECTS], players: &mut [Player; MAX_PLAYERS]) {
    for slot in effects.iter_mut() {
        let Some(effect) = slot else { continue };
        effect.age += 1;
        if effect.age >= effect.life {
            *slot = None;
            continue;
        }
        apply_effect(*effect, players);
    }
    for p in players.iter_mut() {
        p.slowed = p.slowed.saturating_sub(1);
    }
}

fn apply_effect(effect: Effect, players: &mut [Player; MAX_PLAYERS]) {
    let radius = t::body_radius();
    let height = t::body_height();
    for (i, p) in players.iter_mut().enumerate() {
        // An effect never touches the fighter who made it. A fire pillar you
        // cannot stand next to is a fire pillar you cannot use.
        if i as u8 == effect.owner || p.health <= 0 {
            continue;
        }
        match effect.kind {
            EffectKind::FirePillar => {
                let (base, column) = effect.pillar_volumes();
                let caught = base.contains(effect.pos, p.pos, radius, height)
                    || column.contains(effect.pos, p.pos, radius, height);
                if caught && effect.ticks_now() {
                    p.health = (p.health - t::pillar_damage()).max(0);
                }
            }
            EffectKind::BlackSpike => {
                let flat = V3::new(
                    p.pos.x.sub(effect.pos.x),
                    Fx::ZERO,
                    p.pos.z.sub(effect.pos.z),
                )
                .flat_len();
                if flat.raw() <= effect.field_radius().add(radius).raw() {
                    // Drain *and* slow: the field punishes standing in it and
                    // makes leaving it slow, which is what turns a damage
                    // puddle into a positioning tool.
                    p.slowed = t::slow_frames();
                    if effect.ticks_now() {
                        p.health = (p.health - t::spike_drain()).max(0);
                    }
                }
            }
        }
    }
}

/// Keep a grabbed fighter at their captor's arm's length.
fn drag_the_held(players: &mut [Player; MAX_PLAYERS]) {
    let snapshot = *players;
    for victim in players.iter_mut() {
        let Action::Held { .. } = victim.action else {
            continue;
        };
        let by = victim.held_by as usize;
        if by >= MAX_PLAYERS {
            continue;
        }
        let holder = &snapshot[by];
        let reach = t::body_radius().add(t::body_radius());
        victim.pos.x = holder.pos.x.add(holder.facing.x.mul(reach));
        victim.pos.z = holder.pos.z.add(holder.facing.z.mul(reach));
        victim.pos.y = holder.pos.y;
        victim.vel = V3::ZERO;
        victim.grounded = holder.grounded;
    }
}

/// Put an effect into the world, replacing the oldest if the board is full.
fn spawn_effect(effects: &mut [Option<Effect>; MAX_EFFECTS], kind: EffectKind, owner: u8, pos: V3) {
    let life = match kind {
        EffectKind::FirePillar => t::pillar_life(),
        EffectKind::BlackSpike => t::spike_life(),
    };
    let effect = Effect {
        kind,
        owner,
        pos,
        age: 0,
        life,
    };
    if let Some(free) = effects.iter_mut().find(|s| s.is_none()) {
        *free = Some(effect);
        return;
    }
    // Full. Your oldest goes -- **yours**, never the other fighter's. Spamming
    // should cost you your own setup and nothing else; an eviction that reached
    // across owners would mean one fighter could delete the other's by holding
    // a button, which is not a decision anybody made.
    let oldest = effects
        .iter()
        .enumerate()
        .filter_map(|(i, e)| e.filter(|e| e.owner == owner).map(|e| (i, e.age)))
        .max_by_key(|(_, age)| *age)
        .map(|(i, _)| i);
    if let Some(i) = oldest {
        effects[i] = Some(effect);
    }
}

/// Walking speed after whatever is slowing you.
///
/// A slow is a positioning tool, not a damage one: the Blood mage's field hurts
/// while you stand in it and makes leaving it take longer, which is what turns
/// a puddle of damage into a wall.
fn dragged(p: &Player, speed: Fx) -> Fx {
    if p.slowed == 0 {
        return speed;
    }
    speed.mul(t::spike_slow())
}
