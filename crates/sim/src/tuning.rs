//! Every number that decides how the game feels, in one place.
//!
//! These will be changed constantly and over a long period. Two rules make that
//! survivable:
//!
//! 1. **Each value carries its provenance** — why it is what it is, or an
//!    explicit note that it was a guess. A number nobody can justify is a
//!    number nobody dares change.
//! 2. **`docs/design/feel-log.md` records the experiments.** The value lives
//!    here; the reasoning lives there. Reverted experiments are the valuable
//!    ones, so they get logged too.
//!
//! `crates/sim/tests/feel.rs` pins the *relationships* between these, not the
//! values. Move a number freely; if a relationship test fails, either it is a
//! bug or a design decision changed and the documents need updating with it.

use crate::fixed::Fx;
use crate::oven::{self, Scalar};

// ---------------------------------------------------------------------------
// Movement
// ---------------------------------------------------------------------------

/// Walk speed. Everything else is spaced against this, so it is the first
/// number to get right and the most expensive one to change later.
pub fn move_speed() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::MoveSpeed))
}

/// Guarding is a crawl. Blocking should not be a way to travel.
pub fn guard_move_speed() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::GuardMoveSpeed))
}

/// Crouching is slower than walking, so ducking an overhead costs tempo.
pub fn crouch_move_speed() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::CrouchMoveSpeed))
}

/// Takeoff speed, and the gravity it is chosen against.
///
/// Deliberately floatier than a first pass would pick. Verticality is meant to
/// be part of the positioning game, and a jump you are only airborne for a third
/// of a second in is one nobody has time to *do* anything with -- you commit,
/// and it is over before you have read the situation you jumped into. A beginner
/// needs long enough in the air to notice where the other player went.
///
/// Full hop apexes around 2.2 m over roughly a second; a short hop is about half
/// that. Both are relationships the feel tests pin, not numbers to trust.
pub fn jump_speed() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::JumpSpeed))
}
pub fn gravity() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::Gravity))
}

/// Terminal velocity. Without one, a long fall arrives faster than anyone can
/// react to, and per-class fall speed stops meaning anything at the bottom.
pub fn fall_cap() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::FallCap))
}

/// Gravity multiplier while the jump button is still held and you are rising.
///
/// This is what makes jump height variable: hold for a taller jump, tap for a
/// short one, and everything in between. It is a *sustain* rather than a cut on
/// release, because a cut makes the short hop feel like the jump was taken away
/// from you, whereas a sustain makes the tall one feel earned.
pub fn jump_hold_gravity() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::JumpHoldGravity))
}

/// How long the sustain can last. Beyond this, gravity is gravity.
pub fn jump_hold_frames() -> u16 {
    oven::scalar(Scalar::JumpHoldFrames) as u16
}

/// Air acceleration, Quake-style. See `state::air_accelerate`.
pub fn air_accel() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::AirAccel))
}

/// Ceiling on horizontal air speed, as a multiple of the ground walk.
///
/// A deliberate divergence from Source, where strafing gains speed without
/// bound and that unboundedness became the genre. In a fighter built on spacing,
/// a player who can reach any part of the arena from any other has removed
/// spacing from the game. Set high enough that good strafing is rewarded.
pub fn air_speed_cap() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::AirSpeedCap))
}

/// Turn rate toward the opponent, per tick, as a fraction of remaining error.
pub fn turn_rate() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::TurnRate))
}
/// Slower while guarding. This is what makes the facing arc a real cost and
/// what lets an opponent walk around a turtle.
pub fn guard_turn_rate() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::GuardTurnRate))
}

// ---------------------------------------------------------------------------
// Defence
// ---------------------------------------------------------------------------

/// Cosine of the guard arc half-angle. 0.5 is a 120-degree frontal arc.
/// Guard is an arc, not a bubble -- see `defense.md`.
pub fn guard_arc_cos() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::GuardArcCos))
}

/// Frames at the start of a guard that parry instead of blocking.
///
/// **The most important open number in the game.** Human reaction is roughly
/// 15 frames at 60 Hz, so four frames makes parry a read rather than a
/// reaction, which is intended. Whether it is *findable* is the question.
pub fn parry_window() -> u16 {
    oven::scalar(Scalar::ParryWindow) as u16
}

/// What a successful parry costs the attacker. Must be long enough that the
/// punish is worth the risk of trying to parry at all.
pub fn parry_stagger() -> u16 {
    oven::scalar(Scalar::ParryStagger) as u16
}

/// Dodge: committed, directional, invulnerable only at the start.
pub fn dodge_frames() -> u16 {
    oven::scalar(Scalar::DodgeFrames) as u16
}
/// Fewer than DODGE_FRAMES, so the tail is punishable. If these were equal,
/// dodge would beat everything and never be punished.
pub fn dodge_iframes() -> u16 {
    oven::scalar(Scalar::DodgeIframes) as u16
}
pub fn dodge_speed() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::DodgeSpeed))
}

/// How far a crouch lowers the hurtbox. Currently only matters as a flag, since
/// overheads are decided by move property rather than geometry.
pub fn crouch_height_scale() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::CrouchHeightScale))
}

// ---------------------------------------------------------------------------
// Bodies and the arena
// ---------------------------------------------------------------------------

pub fn body_radius() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::BodyRadius))
}
pub fn body_height() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::BodyHeight))
}

/// Knockback decay per tick while stunned. Below 1.0 or a hit sends you
/// sliding forever.
pub fn knockback_decay() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::KnockbackDecay))
}

// ---------------------------------------------------------------------------
// Match
// ---------------------------------------------------------------------------

/// Target time to kill in versus is about 60 seconds. With the damage numbers
/// in `moves.rs` that is roughly six committed hits or seventeen pokes --
/// untested against a real match.
pub fn max_health() -> i32 {
    oven::scalar(Scalar::MaxHealth)
}

/// Pause after a knockout before the next round starts.
pub fn round_over_frames() -> u16 {
    oven::scalar(Scalar::RoundOverFrames) as u16
}

/// Reaction time at 60 Hz, used by the feel tests to reason about what is and
/// is not reactable. Roughly 250 ms, which is the usual figure for a simple
/// visual reaction.
pub const HUMAN_REACTION_FRAMES: u16 = 15;

/// The airdodge: shorter and slower than the grounded one.
///
/// Shorter because you are already committed by being in the air -- the jump
/// was the commitment, and stacking a long vulnerable tail on top of it would
/// make any anti-air a guaranteed kill. Slower because a horizontal burst at
/// ground-dodge speed, from a standing jump, crosses more of the arena than a
/// dodge should.
pub fn air_dodge_frames() -> u16 {
    oven::scalar(Scalar::AirDodgeFrames) as u16
}
pub fn air_dodge_speed() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::AirDodgeSpeed))
}

/// Percent of walking speed kept while throwing a fast poke.
///
/// Pokes are the neutral tool and get thrown constantly. Rooting you for every
/// one made neutral sticky and read as the game snatching the controls away --
/// reported as jarring, and it was. Slowing you keeps the cost without the
/// lurch: you still cannot close or escape at full speed while swinging.
///
/// Sits between the crouch walk and the free walk, so "slowed by swinging" is a
/// speed the player already has a feel for.
pub fn poke_mobility() -> u8 {
    oven::scalar(Scalar::PokeMobility) as u8
}

/// How much horizontal speed survives each frame of a move that roots you.
///
/// Rooting is correct for the committed moves -- that is what commitment means
/// -- but arriving at rooted in a single frame is a snap from a full walk to
/// nothing, which is the jarring part rather than the rooting itself. Over
/// about four frames this bleeds off the speed instead. The distance slid is a
/// few centimetres; it changes nothing about the spacing and everything about
/// how it reads.
pub fn attack_root_decay() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::AttackRootDecay))
}

/// How much vertical speed survives each frame of an aerial's hang.
///
/// The first version deleted it outright, which read as the game snatching the
/// jump away mid-rise. Slowing the momentum instead keeps the extra control
/// over jump height that attacking gives you, without the lurch: you feel the
/// rise bleed off rather than stop.
pub fn air_stall_damp() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::AirStallDamp))
}

/// A small shove in the direction you are holding, when a poke is thrown in the
/// air.
///
/// The basic attack is the one you throw constantly, so this is what makes
/// attacking part of air movement rather than a pause in it -- the hang gives
/// it float, the shove gives it punch. Clamped by the air speed cap like any
/// other air movement, so it cannot be chained into crossing the arena.
pub fn air_attack_boost() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::AirAttackBoost))
}

/// Fraction of your upward speed kept when you let go of jump while rising.
///
/// The sustain alone cannot give a short hop worth having. Holding reduces
/// gravity, so the only thing separating a tap from a hold is that multiplier —
/// which means the short hop is *exactly* that fraction of the full one, and a
/// sustain gentle enough to feel good leaves the floor at about two thirds of
/// the ceiling. A platform fighter wants a quarter or less.
///
/// Cutting the velocity on release is the knob that lowers the floor without
/// touching the ceiling: the full hop never releases while rising, so it is
/// untouched, and the short hop scales with the *square* of this because apex
/// goes as velocity squared.
pub fn jump_release_cut() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::JumpReleaseCut))
}

// ---------------------------------------------------------------------------
// Persistent effects
// ---------------------------------------------------------------------------

/// How often a field effect damages. The number in the move table is per tick,
/// not per frame -- a field hitting every frame would do sixty times its listed
/// damage a second.
pub fn effect_tick_frames() -> u16 {
    oven::scalar(Scalar::EffectTickFrames) as u16
}

/// The fire pillar grows. It starts narrow and short, and the two halves grow
/// differently: the base spreads *out* and the column reaches *up*, so the
/// threat to someone on the ground and the threat to someone jumping over
/// arrive on different schedules.
pub fn pillar_base_radius_start() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::PillarBaseRadiusStart))
}
pub fn pillar_base_radius() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::PillarBaseRadius))
}
pub fn pillar_base_height() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::PillarBaseHeight))
}
pub fn pillar_radius_start() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::PillarRadiusStart))
}
pub fn pillar_column_radius() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::PillarColumnRadius))
}
pub fn pillar_height_start() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::PillarHeightStart))
}
pub fn pillar_height() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::PillarHeight))
}
pub fn pillar_life() -> u16 {
    oven::scalar(Scalar::PillarLife) as u16
}

pub fn spike_radius() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::SpikeRadius))
}
pub fn spike_life() -> u16 {
    oven::scalar(Scalar::SpikeLife) as u16
}

/// Movement multiplier while standing in a drain field.
pub fn spike_slow() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::SpikeSlow))
}

pub fn spike_drain() -> i32 {
    oven::scalar(Scalar::SpikeDrain)
}

/// Frames a structure takes to climb out of the ground. Cosmetic: it is earth,
/// so it comes up through the floor rather than appearing in the air.
pub fn structure_rise() -> u16 {
    oven::scalar(Scalar::StructureRise) as u16
}

/// How wide a structure stands. Visual size and, later, what a fire pillar
/// detonates -- structures have no clock, so there is no lifetime beside it.
pub fn structure_radius() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::StructureRadius))
}

/// How long a slow lingers after leaving the field that applied it. Without a
/// tail the slow flickers on the boundary.
pub fn slow_frames() -> u16 {
    oven::scalar(Scalar::SlowFrames) as u16
}

/// Damage a fire pillar deals each tick to anyone inside either of its volumes.
pub fn pillar_damage() -> i32 {
    oven::scalar(Scalar::PillarDamage)
}

// ---------------------------------------------------------------------------
// Class furniture
// ---------------------------------------------------------------------------
//
// Numbers that used to be `const` in the middle of the code that used them.
// They are as much "how this class feels" as any frame count, and being out of
// the Oven's reach meant they could only be changed by a recompile -- and, in
// the body's case, could silently disagree with the knob meant to control them.

/// How fast the Bulwark's shield travels.
pub fn shield_speed() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::ShieldSpeed))
}

/// How far it goes before planting.
pub fn shield_range() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::ShieldRange))
}

pub fn shield_radius() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::ShieldRadius))
}

pub fn shield_damage() -> i32 {
    oven::scalar(Scalar::ShieldDamage)
}

pub fn shield_hitstun() -> u16 {
    oven::scalar(Scalar::ShieldHitstun) as u16
}

pub fn shield_blockstun() -> u16 {
    oven::scalar(Scalar::ShieldBlockstun) as u16
}

pub fn shield_knockback() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::ShieldKnockback))
}

/// Horizontal speed of the leap to a shield in flight -- the Bulwark's approach.
pub fn leap_speed() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::LeapSpeed))
}

pub fn leap_rise() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::LeapRise))
}

/// How far the Reaver may stray from a placed shadow before it snaps back.
pub fn shadow_leash() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::ShadowLeash))
}

pub fn shadow_place_ahead() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::ShadowPlaceAhead))
}

pub fn structure_ahead() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::StructureAhead))
}

pub fn structure_height() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::StructureHeight))
}

/// What a dodge keeps of its speed each frame.
pub fn dodge_decay() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::DodgeDecay))
}

/// What knockback keeps each frame. Decay rather than a dead stop, so being hit
/// mid-jump is not immediately steered out of.
pub fn stun_decay() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::StunDecay))
}

// ---------------------------------------------------------------------------
// Stride
// ---------------------------------------------------------------------------
//
// How much ground one full cycle of a walk or a run covers. These are the
// numbers that decide whether the feet skate, and they are shared three ways:
// the simulation advances the stride phase with them, the renderer plays the
// clips at that phase, and the clips themselves are authored by placing every
// planted foot with the same arithmetic. One copy, here, or the feet slide.
//
// They are not free choices. A leg is 0.87 m long and a hip is 0.86 m off the
// floor at contact, so a foot can be at most about 0.34 m ahead of the hip
// before the leg runs out -- and stride length follows from that.

/// Speed at which the walk cycle is at full weight. The guarding walk speed is
/// two, and that is the speed a fighter spends a tense exchange at.
pub const WALK_AT: Fx = Fx::ratio(22, 10);
/// And where the run is. Seven metres per second is a run by any honest
/// measure, whatever the movement knob is called.
pub const RUN_AT: Fx = Fx::ratio(7, 1);

pub const WALK_STRIDE: Fx = Fx::ratio(110, 100);
pub const RUN_STRIDE: Fx = Fx::ratio(270, 100);
pub const CROUCH_STRIDE: Fx = Fx::ratio(80, 100);
/// A sidestep covers less ground per cycle than a stride forward, because a leg
/// swung sideways runs out of hip long before one swung forward runs out of leg.
pub const STRAFE_STRIDE: Fx = Fx::ratio(72, 100);

/// What a body keeps while the round-over pause runs.
pub fn settle_decay() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::SettleDecay))
}

/// Reach, damage and recovery multipliers for one of the Champion's forms.
/// Three kits from one move table, so these nine numbers are most of the class.
pub fn form_modifiers(form: crate::class::Form) -> (Fx, Fx, Fx) {
    use crate::class::Form;
    let raw = |s| Fx::from_raw(oven::scalar(s));
    match form {
        Form::Hammer => (
            raw(Scalar::HammerReach),
            raw(Scalar::HammerDamage),
            raw(Scalar::HammerRecovery),
        ),
        Form::Sword => (
            raw(Scalar::SwordReach),
            raw(Scalar::SwordDamage),
            raw(Scalar::SwordRecovery),
        ),
        Form::Spear => (
            raw(Scalar::SpearReach),
            raw(Scalar::SpearDamage),
            raw(Scalar::SpearRecovery),
        ),
    }
}

pub fn meter_max() -> i32 {
    oven::scalar(Scalar::MeterMax)
}

pub fn meter_deep() -> i32 {
    oven::scalar(Scalar::MeterDeep)
}

pub fn meter_burn() -> i32 {
    oven::scalar(Scalar::MeterBurn)
}

// ---------------------------------------------------------------------------
// Stones
// ---------------------------------------------------------------------------
//
// A structure stopped being a marker and became a solid, so it needs the
// numbers a solid needs: what it does to what is in its way, and what it costs
// to be standing on the spot when one arrives. See `crate::stones`.

/// How far out of the ground a stone has to be before it counts as erupting
/// rather than churning.
///
/// A fraction of the rise rather than a frame count, so it is the *curve* that
/// decides when the telegraph ends. Reshape the rise and the warning moves with
/// it; pick a frame instead and the two drift apart the first time anybody
/// drags a handle.
pub fn stone_erupt() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::StoneErupt))
}

/// Movement multiplier while a stone is churning under your feet.
///
/// Slight on purpose. This is the telegraph made tactile -- you feel the ground
/// go before you see the stone -- and a telegraph that also stops you moving
/// would make the eruption unavoidable rather than readable.
pub fn stone_churn_slow() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::StoneChurnSlow))
}

/// What the eruption costs whoever stood on it. Small: the stagger is the
/// point, and the damage is there so that ignoring a telegraph is never free.
pub fn stone_erupt_damage() -> i32 {
    oven::scalar(Scalar::StoneEruptDamage)
}

pub fn stone_erupt_stagger() -> u16 {
    oven::scalar(Scalar::StoneEruptStagger) as u16
}

/// How much of a rising surface's speed whatever is riding it keeps.
///
/// The eruption's own climb is what throws a stone -- or a fighter -- off the
/// top of another, so this is the knob that decides whether a stone raised
/// underneath is a lift or a launch. One, and a rider leaves at exactly the
/// speed the surface was climbing at.
pub fn stone_lift() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::StoneLift))
}

/// Fraction of the closing speed a stone hands to whatever it runs into.
pub fn stone_knock_handed() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::StoneKnockHanded))
}

/// What both stones keep of their speed after a knock. Below one, so the
/// exchange costs them both and a stone cannot bowl through a row of them.
pub fn stone_knock_damp() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::StoneKnockDamp))
}

/// What a resting stone keeps of its speed each frame. This is what brings a
/// knocked stone to a stop somewhere short of the far wall.
pub fn stone_friction() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::StoneFriction))
}

/// The curve a structure climbs out of the ground on.
pub fn structure_rise_curve() -> crate::curve::Curve {
    crate::curve::Curve {
        x1: Fx::from_raw(oven::scalar(Scalar::RiseCurveX1)),
        y1: Fx::from_raw(oven::scalar(Scalar::RiseCurveY1)),
        x2: Fx::from_raw(oven::scalar(Scalar::RiseCurveX2)),
        y2: Fx::from_raw(oven::scalar(Scalar::RiseCurveY2)),
    }
}

// ---------------------------------------------------------------------------
// The Ridgeback
//
// See `docs/design/monsters.md`. Three groups, and they are edited separately
// because they answer different questions: how the animal is built, how it
// decides, and what it takes to stay on its back.
// ---------------------------------------------------------------------------

/// Multiplies every part box. The one number you actually reach for while
/// playing -- *how big is it* -- which is why the proportions in
/// `monster::SHAPES` are a constant and this is not.
pub fn monster_scale() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::MonsterScale))
}

/// Pool health. The per-part vulnerabilities below are what decide how long a
/// fight actually takes, so this is the *second* number to reach for.
pub fn monster_health() -> i32 {
    oven::scalar(Scalar::MonsterHealth)
}

/// Health of a breakable limb. Breaking a foreleg costs it the turn toward
/// that side, which is the one consequence in the fight a player can point at.
pub fn limb_health() -> i32 {
    oven::scalar(Scalar::LimbHealth)
}

/// How far from the arena wall it is kept. A creature nine metres long in a
/// twenty-eight metre arena needs somewhere to stand.
pub fn monster_margin() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::MonsterMargin))
}

/// Walking, backing off, and how quickly it changes between them. Below the
/// player's own walk on purpose: it catches you by cornering you, not by
/// outrunning you.
pub fn monster_walk() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::MonsterWalk))
}
pub fn monster_back() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::MonsterBack))
}
pub fn monster_accel() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::MonsterAccel))
}

/// The distance it tries to hold, and how hard it corrects toward it. Set near
/// the middle of the move set's range band so that most of what it wants to do
/// is available most of the time.
pub fn prowl_range() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::ProwlRange))
}
pub fn approach_gain() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::ApproachGain))
}

/// Turning, in turns per second and turns per second squared.
///
/// The rate cap is how fast it can come round. The acceleration cap is what
/// gives it mass -- and it is the more interesting of the two, because it is
/// what produces the overshoot a player cuts back across. See
/// `monster::Monster::steer`.
pub fn turn_rate_max() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::TurnRateMax))
}
pub fn turn_gain() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::TurnGain))
}
pub fn turn_accel() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::TurnAccel))
}
/// What a broken foreleg does to the turn toward that side.
pub fn turn_hurt() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::TurnHurt))
}
/// How a swing in progress bleeds off once a move commits. Stopping dead
/// visibly clamps the animal mid-turn.
pub fn turn_settle() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::TurnSettle))
}

/// **The difficulty model, both halves.**
///
/// `glance_frames` is how stale its information is: it takes one sample of the
/// target and works from it until the next. `lead` is how much of the
/// extrapolation from that sample it trusts, as a fraction of the move's own
/// startup -- so a slow lunge leads further, which is both correct and one knob
/// instead of one per move.
///
/// Short glance with a long lead is frightening. Long glance with no lead is an
/// animal you can walk around. The skill the pair rewards is specific: change
/// direction between its glances.
pub fn glance_frames() -> u16 {
    oven::scalar(Scalar::GlanceFrames) as u16
}
pub fn lead() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::Lead))
}
/// The horizon it leads by while merely walking toward you, in frames.
pub fn prowl_lead() -> u16 {
    oven::scalar(Scalar::ProwlLead) as u16
}

/// The pause between moves. Without one it is a chain gun.
pub fn think_frames() -> u16 {
    oven::scalar(Scalar::ThinkFrames) as u16
}

/// How close to the best a move has to score to make the draw, as a percentage.
/// A hundred always throws the best move and is therefore a script you can
/// memorise; zero is noise. In between, the distribution is learnable and the
/// next move is not.
pub fn decisiveness() -> i32 {
    oven::scalar(Scalar::Decisiveness)
}

/// How much harder it commits as it loses health, and how much it dislikes
/// repeating itself and for how long.
pub fn hurt_aggression() -> i32 {
    oven::scalar(Scalar::HurtAggression)
}
pub fn variety_penalty() -> i32 {
    oven::scalar(Scalar::VarietyPenalty)
}
pub fn variety_frames() -> u16 {
    oven::scalar(Scalar::VarietyFrames) as u16
}

/// Ridge damage that puts it on the ground, and how fast the pool comes back.
/// Regeneration is what stops a topple being saved up across a whole fight.
pub fn poise_max() -> i32 {
    oven::scalar(Scalar::PoiseMax)
}
pub fn poise_regen() -> i32 {
    oven::scalar(Scalar::PoiseRegen)
}
pub fn topple_frames() -> u16 {
    oven::scalar(Scalar::ToppleFrames) as u16
}
/// Going down is fast and getting up is not.
pub fn topple_fall() -> u16 {
    oven::scalar(Scalar::ToppleFall) as u16
}
pub fn topple_rise() -> u16 {
    oven::scalar(Scalar::ToppleRise) as u16
}
pub fn topple_pitch() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::TopplePitch))
}
pub fn topple_drop() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::ToppleDrop))
}

/// A flinch interrupts it, and only a hit that hurts causes one -- which in
/// practice means a ridge hit, because the armour elsewhere keeps the number
/// below the threshold. That rule is emergent rather than written down, and it
/// is the better for it.
pub fn flinch_frames() -> u16 {
    oven::scalar(Scalar::FlinchFrames) as u16
}
pub fn flinch_pitch() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::FlinchPitch))
}
pub fn flinch_threshold() -> i32 {
    oven::scalar(Scalar::FlinchThreshold)
}

/// What fraction of an attack's damage each part passes through. Below one is
/// armour. The ridge is above one, and that difference is the entire reason to
/// climb.
pub fn vuln_head() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::VulnHead))
}
pub fn vuln_neck() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::VulnNeck))
}
pub fn vuln_barrel() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::VulnBarrel))
}
pub fn vuln_ridge() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::VulnRidge))
}
pub fn vuln_tail() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::VulnTail))
}
pub fn vuln_tail_tip() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::VulnTailTip))
}
pub fn vuln_foreleg() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::VulnForeleg))
}
pub fn vuln_hindleg() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::VulnHindleg))
}

// The pose amplitudes. These are not decoration: the pose is what a rider is
// standing on, so every one of them is also a number that decides whether a
// move throws people off. See `monster::pose_of`.

pub fn bite_draw() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::BiteDraw))
}
pub fn bite_reach() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::BiteReach))
}
pub fn bite_rear() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::BiteRear))
}
pub fn stomp_lift() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::StompLift))
}
pub fn stomp_drop() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::StompDrop))
}
pub fn stomp_bob() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::StompBob))
}
pub fn sweep_wind() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::SweepWind))
}
pub fn sweep_swing() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::SweepSwing))
}
/// How much the body answers the tail. A tail that heavy cannot swing without
/// the rest of the animal paying for it, and the payment is what shears anyone
/// standing on the barrel.
pub fn sweep_counter() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::SweepCounter))
}
pub fn charge_lean() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::ChargeLean))
}
pub fn charge_gallop() -> i32 {
    oven::scalar(Scalar::ChargeGallop)
}
pub fn charge_bounce() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::ChargeBounce))
}
pub fn slam_rear() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::SlamRear))
}
pub fn slam_rise() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::SlamRise))
}
pub fn slam_dip() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::SlamDip))
}
/// Fractional on purpose: whether the shake ends where it started decides
/// whether it reads as a shudder or as a swerve.
pub fn shake_cycles() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::ShakeCycles))
}
pub fn shake_yaw() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::ShakeYaw))
}
pub fn shake_pitch() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::ShakePitch))
}
pub fn shake_ramp() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::ShakeRamp))
}

// ---------------------------------------------------------------------------
// Riding
// ---------------------------------------------------------------------------

/// How close a falling body has to be to a mountable face to land on it.
pub fn mount_snap() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::MountSnap))
}

/// How far over an edge you may stand before you are standing on nothing, as a
/// fraction of your own width.
pub fn edge_grace() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::EdgeGrace))
}

/// **Grip: the acceleration a rider can hold on through**, in metres per second
/// squared, and what bracing multiplies it by.
///
/// Every buck in the game is this one comparison. Nothing tags a move "throws
/// riders" -- a move that moves the surface hard enough throws whoever is on
/// it, and a move that does not, does not. Crouching braces, which is a third
/// answer alongside dodging and leaving.
///
/// For scale: an ordinary walk or turn is tens, a stomp is a couple of hundred,
/// and the slam's reversal is over a thousand.
pub fn grip() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::Grip))
}
pub fn brace_grip() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::BraceGrip))
}

/// What being thrown does: outward along the surface, upward off it, and how
/// long you spend unable to answer for it.
pub fn throw_kick() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::ThrowKick))
}
pub fn throw_lift() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::ThrowLift))
}
pub fn throw_stun() -> u16 {
    oven::scalar(Scalar::ThrowStun) as u16
}

/// Frames after landing before the grip test starts. You get a moment to plant
/// your feet, and without it the first frame aboard reads as an infinite
/// acceleration and throws you straight back off.
pub fn mount_settle() -> u16 {
    oven::scalar(Scalar::MountSettle) as u16
}

/// Walking speed on the creature's back, as a fraction of the ground walk.
/// Slower, because the footing is not flat and because a back you can cross in
/// half a second is not a place you have to hold.
pub fn rider_speed() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::RiderSpeed))
}

/// Where the two sides start a hunt, measured out from the arena's centre.
///
/// Far enough apart that the opening of a fight is an approach rather than an
/// ambush: both sides get to read the other before anything is committed.
pub fn monster_spawn() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::MonsterSpawn))
}
pub fn hunter_spawn() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::HunterSpawn))
}

/// How big a step up a rider can simply walk up.
///
/// The creature is terrain, and terrain has steps: the tail sits two thirds of
/// a metre below the back, and without this the only way from one to the other
/// is a jump nobody would think to try. A rider who walks into a mountable face
/// this close above their feet is put on top of it instead of stopped by it,
/// which is what every platformer does and what makes an animal feel like
/// somewhere you can move around rather than a collection of ledges.
pub fn step_up() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::StepUp))
}
