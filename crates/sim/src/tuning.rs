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
/// **Cut from 17.7 to 16.2 on 2026-09-17**, taking about a third off every
/// apex in the game. Apex goes as the *square* of this and airtime goes as this,
/// so the height came down by a third while the hang only lost a fifth -- which
/// is why the height is tuned here rather than by leaning on gravity. A full
/// hop now runs from 2.7 m on the Bulwark to 6.0 m on the Dual mage, over 0.7 s
/// to 1.2 s.
///
/// The reason is that a jump is the *floor* of this game's movement system
/// rather than the whole of it. The Elementalist rides a structure up and the
/// Champion vaults, and both were being reached to within a quarter by pressing
/// space. See `docs/design/feel-log.md`.
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
///
/// **Raised from 0.56 to 0.72 on 2026-09-17, with the window below cut from 26
/// frames to 18.** Together those two were what made a held jump read as an
/// elevator. At 0.56 for 26 frames the rise lost only half its speed over four
/// tenths of a second and covered five of its six metres doing it, so the climb
/// looked like a constant one and gravity looked like something that switched
/// on at the top. A sustain has to be a *discount* on gravity rather than a
/// suspension of it, and you have to be able to see the rise slowing the whole
/// way up. See `docs/design/feel-log.md`.
pub fn jump_hold_gravity() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::JumpHoldGravity))
}

/// How long the sustain can last. Beyond this, gravity is gravity.
///
/// Long enough to be a real decision, short enough that most of the rise
/// happens under ordinary gravity -- see [`jump_hold_gravity`] for why those
/// are the two things it is balanced between.
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

/// How long a move you just threw is unavailable to you.
///
/// **This is not a cooldown, and the difference is the whole of why it is
/// allowed to exist** -- see `docs/design/combat-kernel.md`. A cooldown asks
/// *did I have it available*, which it can ask because it takes a move away
/// from you while you are doing something else. This only ever locks the move
/// you have just this moment thrown, so the answer is always "yes, everything
/// else in the kit": it does not gate access, it charges for *repetition*.
///
/// Thirty frames is half a second, and it is measured from the frame the move
/// comes out rather than from the end of its recovery. That is what keeps it
/// honest: it can only ever spend frames you would otherwise have had free, so
/// a move that already commits you for longer than this never notices it at
/// all. Every committed heavy in the game is in that group, and every auto and
/// fast poke is not -- which is exactly the set the rule is aimed at. The
/// relationship is pinned by `feel::the_lockout_only_taxes_the_cheap_moves`.
///
/// A guess, and flagged as one: 30 is the first number, not a measured one.
pub fn repeat_lockout() -> u16 {
    oven::scalar(Scalar::RepeatLockout) as u16
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

/// How high above the feet an ability comes out of.
///
/// **The single point the whole aiming scheme is hung on.** It is where a
/// skillshot starts, it is what the range sphere is centred on, and it is the
/// point the camera orbits -- so the line the crosshair draws in space and the
/// line the ability travels are the same line. Move it and all three move
/// together, which is the only way they can stay honest.
///
/// Chest height on a fighter who is `body_height` tall: hands, not eyes. Eye
/// height would put the origin where the camera is and read as a first-person
/// shot fired from a third-person body.
pub fn cast_height() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::CastHeight))
}

/// How far to one side of the centre line a hand is.
///
/// One number for the whole roster, like the body radius, and for the same
/// reason: the simulation does not know how broad any particular fighter's
/// shoulders are drawn -- that is a build in `view::skeleton` -- and a hit
/// volume that changed size with the model would make the same move a different
/// move on six characters.
///
/// It matters for exactly one thing, and it is worth being precise about which:
/// **where a one-armed move leaves from.** See `crate::aim::hand_origin`. A
/// punch that came out of the sternum would put the Dual mage's dark and light
/// autos in the same place, and which of the two just landed is the whole of
/// how her meter is steered.
pub fn hand_offset() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::HandOffset))
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

/// Percent of walking speed kept while throwing a committed move.
///
/// **Nothing roots any more.** Committed moves used to set this to zero, which
/// is what commitment used to mean: you stopped, and the stick did nothing
/// until the move was over. It read as jarring for exactly the reason the poke
/// did -- a character who ignores the input is the game taking the controls
/// away -- and the answer is the same one, further down. A crawl.
///
/// Below the guard walk, which is the slowest thing you can otherwise choose to
/// do, so a committed move is still the most your feet ever cost you. What
/// commitment means now is the other half of it: you cannot jump, dodge, guard
/// or throw anything else until the move is finished, and that was always the
/// part doing the work. See `docs/design/controls.md`.
pub fn committed_mobility() -> u8 {
    oven::scalar(Scalar::CommittedMobility) as u8
}

/// How much horizontal speed survives each frame of a move that hinders you.
///
/// A move takes your feet away in proportion to how much it commits you, but
/// *arriving* at the hindered speed is a ramp rather than an assignment. The
/// snap from a full walk to nothing in a single frame was the jarring half of
/// the old dead stop, and dropping straight to a crawl instead would have put
/// the same lurch back with a different number at the bottom of it. Over about
/// four frames this bleeds down to whatever the move allows.
///
/// It is also what stops you dead when you let the stick go mid-move: the floor
/// of the ramp is the move's own speed while you are steering, and zero when
/// you are not.
pub fn hindrance_decay() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::HindranceDecay))
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

// ---------------------------------------------------------------------------
// The Blood mage's movement, both of it behind a flag
// ---------------------------------------------------------------------------
//
// **This class has no movement at all and cannot stay that way.** Everything
// else in the roster has one thing it does with the ground: the Reaver crosses
// to her shadow, the Elementalist rides a structure up, the Champion vaults,
// the Dual mage floats, the Bulwark leaps to its shield. The Blood mage walks.
// In a game where jumping was just cut back specifically so that the class
// techniques matter more, walking is not a position to be in.
//
// Two answers are built, both off a flag, because which one is the class is a
// question for somebody playing it rather than for this file:
//
//   * **The Grasp haul.** Four arms that converge on a wall or on the creature
//     rather than on a person pull *her* instead, which is a grappling hook
//     made out of an ability she already has. It is the one with a thematic
//     tie -- the arms are already a thing that closes distance, and this is
//     the same sentence with the subject swapped -- and it costs her a whiffed
//     Grasp's worth of health to use as movement.
//   * **The blink dodge.** Her dodge covers a long flat distance at once
//     instead of a short one over twenty-two frames. Honest about what it is:
//     there is no thematic tie, and it leans entirely on the animation to make
//     it feel like it belongs.
//
// The combinations are the point. Both on is a very mobile Blood mage and
// probably too much; neither is where she is today.

/// Does a Grasp that catches nothing but scenery pull her to it?
pub fn grasp_hauls() -> bool {
    oven::scalar(Scalar::GraspHauls) != 0
}

/// How fast the haul reels her in.
///
/// Constant while it runs, like the Reaver's dash and for the same reason: the
/// distance is whatever the Grasp's own reach and hold chose, so a decaying
/// pull would cover a distance that depends on how the decay happens to be
/// tuned rather than on the thing the player aimed.
///
/// Constant *speed* rather than constant time, which is the opposite of the
/// blink below and right for the opposite reason: you are being reeled in on a
/// rope, so a short pull is quick and a long one is a journey. That difference
/// is the whole of why `Player::haul_speed` is a field and not a knob.
pub fn grasp_haul_speed() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::GraspHaulSpeed))
}

/// Is her dodge a blink?
pub fn dodge_blinks() -> bool {
    oven::scalar(Scalar::DodgeBlinks) != 0
}

/// How far the blink goes.
///
/// Longer than a dodge covers, which is the whole of the idea -- an ordinary
/// dodge is `dodge_speed` decaying over `dodge_frames` and lands a little over
/// four metres away. **Flat**, and deliberately: what this class is missing is
/// ground, not height, and a blink that also went up would be a second jump on
/// the one class that has not earned one.
pub fn blink_range() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::BlinkRange))
}

/// How many frames the blink takes to cross that distance.
///
/// Not zero. A teleport that resolves inside one frame cannot be read by the
/// person opposite -- she is simply somewhere else -- and it cannot be drawn
/// either, which on the answer with no thematic tie is the only thing it has
/// going for it. A handful of frames is a smear rather than a cut, and it
/// still lands well inside the dodge's own invulnerability.
pub fn blink_frames() -> u16 {
    oven::scalar(Scalar::BlinkFrames).max(1) as u16
}

/// How much faster the Dual mage moves once she is deep or ascended.
///
/// The one thing on this class that depth moves which is **not** a force.
/// Everything else on the curve is how hard she hits and how big it is; this is
/// how fast she gets there, and it is a step rather than a curve because it is
/// answering a yes-or-no question -- are her feet on the floor -- rather than a
/// how-much one. See `state::floating`.
pub fn float_move_speed() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::FloatMoveSpeed))
}

/// What each successive aerial hang in one airtime is worth.
///
/// **The air gives you less each time you ask.** A hang costs nothing but the
/// move that carries it, and the repeat lockout only stops one move being
/// thrown twice -- so a class with two interchangeable pokes can alternate them
/// and simply not come down. The Dual mage is that class by construction: her
/// two autos are the same punch mirrored, and they are how she steers her
/// meter, so she throws them alternately as a matter of course.
///
/// Compounding rather than a hard cap, because a cap has an edge somebody finds
/// and plays against, and this has none: the sum of every hang an airtime can
/// contain is `first / (1 - this)`, which is bounded however long you stay up.
/// Reset on landing, like the airdodge.
pub fn air_stall_falloff() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::AirStallFalloff))
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
///
/// Trimmed to 0.52 on 2026-09-17 to hold the short hop at a quarter of the full
/// one after the sustain was shortened. A stiffer sustain takes more off the
/// full hop than off the tap, so without this the two would have drifted back
/// toward each other.
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

/// Frames a structure takes to climb out of the ground.
///
/// It was described here as cosmetic -- it is earth, so it comes up through the
/// floor rather than appearing in the air -- and that stopped being true when a
/// stone became a solid you can stand on. **How long the rise takes is how fast
/// the eruption is**, because the burst at the end of the curve is a real
/// surface speed that a rider keeps ([`stone_lift`]), and it is therefore the
/// knob that decides how high a structure jump goes.
///
/// **Left at 14, and that is a decision rather than an oversight.** It was
/// moved to 16 and then 17 on 2026-09-17 to bring the structure jumps down, and
/// put back the next day.
///
/// The chain a structure jump is made of turns out to be a **resonance**
/// between how fast she rises and how fast the stone grows: the eruption has to
/// outrun her by just enough to catch her feet again, and how many takeoffs
/// each technique gets out of one stone therefore moves around under tuning --
/// and moves *non-monotonically*. Raising her jump can cost the single a link,
/// because she outruns the stone that was going to catch her. Three frames on
/// this knob roughly halved the double while the single barely noticed.
///
/// A knob with that shape is not one to turn on a calculation. If the structure
/// jumps need to come down again, somebody has to play each setting rather than
/// solve for a target. See `docs/design/feel-log.md`.
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

/// The most weight the shield holds, in health: what it can be holding at once.
/// A guess at a little over two committed blows, so a Bulwark who has blocked a
/// string is visibly full and one who blocked a poke is visibly not.
pub fn weight_cap() -> Fx {
    Fx::from_int(oven::scalar(Scalar::WeightCap))
}

/// How long a full shield takes to empty with nothing landing on it. A clock
/// rather than a rate, so the knob reads the way the play script asks the
/// question: ten seconds and the exchange is over.
pub fn weight_drain_frames() -> i32 {
    oven::scalar(Scalar::WeightDrain)
}

/// A parried blow's deposit, as a multiple of its damage. The four-frame read
/// already pays a stagger; this is how much more it pays. Open -- see
/// `bulwark-v2.md`.
pub fn parry_load() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::ParryLoad))
}

/// What each unit of weight adds to Slam's damage. Half, so a full shield's
/// slam gives back half of what it took on top of the blow itself -- enough to
/// be the class's big hit without being a round.
pub fn slam_weight_damage() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::SlamWeightDamage))
}

/// How much wider Slam's shake is from a full shield, in metres; a part-full
/// one adds its share.
pub fn slam_weight_radius() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::SlamWeightRadius))
}

/// What Slam adds per metre per second the Bulwark was falling when it came
/// down -- the scaling the kit sketched, so a slam out of a leap is worth more
/// than one thrown standing. The Champion's spike charges the same way.
pub fn slam_fall_damage() -> i32 {
    oven::scalar(Scalar::SlamFallDamage)
}

/// How full the shield must be for Slam's shake to stagger, as a share of the
/// cap.
pub fn slam_stagger_share() -> Fx {
    Fx::ratio(oven::scalar(Scalar::SlamStaggerShare), 100)
}

/// The stagger a full slam's shake leaves on whoever it catches unguarded.
pub fn slam_full_stagger() -> u16 {
    oven::scalar(Scalar::SlamStaggerFrames) as u16
}

/// A full shield's flight speed, as a share of an empty one's: a loaded throw
/// is a boulder, and a boulder is slower to arrive and easier to read.
pub fn throw_speed_full() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::ThrowSpeedFull))
}

/// What each unit of weight adds to the thrown shield's damage.
pub fn throw_weight_damage() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::ThrowWeightDamage))
}

/// How full a thrown shield must be to knock down what it hits, as a share of
/// the cap.
pub fn knockdown_share() -> Fx {
    Fx::ratio(oven::scalar(Scalar::KnockdownShare), 100)
}

/// How long a loaded throw leaves its victim on the floor.
pub fn knockdown_frames() -> u16 {
    oven::scalar(Scalar::KnockdownFrames) as u16
}

/// The planted shield's size as a wall, empty and full, as multiples of the
/// Elementalist's stone. Between the two by how full it landed.
pub fn wall_size_empty() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::WallSizeEmpty))
}

pub fn wall_size_full() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::WallSizeFull))
}

/// Blocked knockback at a full shield, as a share of it empty. The more it has
/// taken, the less it moves -- the pushback resistance `defense.md` promised.
pub fn heavy_pushback() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::HeavyPushback))
}

/// How far the Reaver may stray from a shadow standing out on the field before
/// it comes and finds her.
///
/// **Longer than the throw**, and it has to be: the shadow arrives at up to the
/// send's own reach, and a leash shorter than that would have it turn round on
/// the frame it landed. The gap between the two is how far she may walk off the
/// line before the line comes after her.
///
/// **Twice the throw since 2026-09-17**, where it used to be a third longer.
/// The shadow is the Reaver's whole movement game -- she sends it somewhere
/// useful and then chooses when to cross to it -- and a leash that close to the
/// throw meant the placement expired while she was still deciding, so the
/// answer to "when do I take this" was usually "now, before it leaves". Twice
/// the throw is room to leave it somewhere and go and do something else.
/// [`shadow_dash_speed`] went up with it, because the dash has to be able to
/// cross whatever this is.
pub fn shadow_leash() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::ShadowLeash))
}

/// How far behind her the attending shadow stands.
///
/// Presentation with teeth: the shadow copies her swings, so where it stands is
/// where its copy lands. Behind her rather than on her, so the two bodies can
/// be told apart at a glance -- which is the whole reason the number exists.
pub fn shadow_trail() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::ShadowTrail))
}

/// How much of the gap the attending shadow closes each frame.
///
/// An ease rather than a hard follow, so it swings out behind her when she
/// turns and drifts back in when she stops. One is welded to her back.
pub fn shadow_follow() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::ShadowFollow))
}

/// How many frames after her the shadow throws the same move.
pub fn shadow_lag() -> u16 {
    oven::scalar(Scalar::ShadowLag).max(0) as u16
}

/// What the shadow's copy of a move deals, as a share of hers.
pub fn shadow_echo() -> Fx {
    Fx::ratio(oven::scalar(Scalar::ShadowEcho), 100)
}

/// Frames the shadow spends flying out to where it was sent.
pub fn shadow_send_frames() -> u16 {
    oven::scalar(Scalar::ShadowSendFrames).max(1) as u16
}

/// How fast the shadow comes home when it is recalled.
pub fn shadow_home_speed() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::ShadowHomeSpeed))
}

/// How much speed the recall leaves whoever it runs through.
pub fn shadow_recall_slow() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::ShadowRecallSlow))
}

/// How near the crosshair has to be to the shadow for a forward dodge to become
/// a dash to it.
///
/// A radius around the shadow, in metres, tested against the crosshair's own
/// ray -- see `aim::pointing_at`. Generous on purpose: the dodge is the
/// class's movement and it must not be lost to a pixel.
pub fn shadow_lock_cone() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::ShadowLockCone))
}

/// How long a right click stays live, waiting for a frame she can spend it on.
///
/// Counted in frames **including the press itself**, so 1 means no memory at
/// all -- the press is read on the frame it happens or not at all, which is how
/// every other button in the game works.
///
/// It exists because the shadow is the one input in the kit that is not an
/// attack. An attack eaten by another move's frames is the game telling you
/// that you were busy, and that is correct. The shadow is the class's *escape*
/// -- from where she is standing and from what she has committed to -- and an
/// escape that only answers on one frame in twenty is not an escape, it is a
/// timing test with the mechanic behind it.
///
/// **Send shadow cuts a recovery short on its own** (see
/// `state::queue_the_shadow`), so the only frames left for this number to
/// cover are the startup and active ones of whatever she is already throwing.
/// That is what sets it, and it is measured rather than guessed: the longest
/// of those is Executioner's sixteen and four, which puts the first
/// cancellable frame twenty-two frames after the move began. Twenty-four
/// carries a press thrown alongside it, with a frame to spare.
///
/// Anything shorter breaks a chain the kit is named for. Twelve was tried
/// first, derived from Slash on the grounds that Slash is the move she throws
/// most -- and it silently dropped `Q` then right click, the lotus drag that
/// the kit document calls the class's biggest turn, because the Guillotine
/// takes sixteen frames to become cancellable. Deriving a buffer from the
/// *commonest* input rather than the *longest wait* is how you get a number
/// that works everywhere except the combo.
///
/// Being hit clears it outright, so the memory never survives a change of
/// situation -- see `shadow::queue_order`. That is what lets it be this long
/// without the shadow ever flying out on a press the player had given up on,
/// which would be worse than dropping one: where the shadow stands is the
/// whole class.
pub fn shadow_buffer() -> u16 {
    oven::scalar(Scalar::ShadowBuffer).max(1) as u16
}

/// How long the carry lasts: the window a dash's arrival opens, in which a jump
/// takes the speed she crossed at up with her.
///
/// **A timing rather than a distance.** What is left of the dodge when she
/// arrives is already a window of exactly this kind -- she is still sliding,
/// and the slide decays -- but its length is however much of the dodge the
/// crossing did not spend, which is a function of how far away she left the
/// shadow. At the end of the leash that is about a frame, and a tech nobody
/// can hit at the range the class is built around is a tech that does not
/// exist. So arriving tops the dodge up to at least this many frames, and the
/// window is the same length however far she came.
///
/// It never *shortens* one: a short dash keeps the whole vulnerable tail it
/// has always had, and only the first frames of that tail are the window.
pub fn shadow_carry() -> u16 {
    oven::scalar(Scalar::ShadowCarry).max(0) as u16
}

// ---------------------------------------------------------------------------
// The Reaver's v2: the shadow aims and keeps a tally, and her hits cash it
// ---------------------------------------------------------------------------
//
// First values, 2026-09-23, all guesses from `docs/design/shadow-reaver-v2.md`
// and none of them played. The feel log is where they get argued about.

/// How far past the copied move's reach the shadow, out on the field, will
/// still turn to a body. Half a metre: enough that a target stepping out of
/// reach during the wind-up is still followed, not so much that a copy turns to
/// somebody it plainly cannot touch.
pub fn shadow_aim_slack() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::ShadowAimSlack))
}

/// Does the shadow, out on the field, turn its copy to a body at all? On. The
/// switch is there so the before and after can be played side by side.
pub fn shadow_aims() -> bool {
    oven::scalar(Scalar::ShadowAims) != 0
}

/// What the **attending** shadow's copy deals, as a share of her swing. Half of
/// the field copy's, from the proposal: with the tally in place, the copy at
/// her heel is the one thing in the kit that argues for standing in melee, and
/// it has to argue less. Twelve rather than twelve and a half because it is a
/// percentage.
pub fn shadow_echo_attending() -> Fx {
    Fx::ratio(oven::scalar(Scalar::ShadowEchoAttending), 100)
}

/// The most marks one body can carry. Five is the proposal's first guess: too
/// low and the burst is a bonus, too high and the shadow never fills it before
/// the victim walks away.
pub fn mark_cap() -> u8 {
    oven::scalar(Scalar::MarkCap).clamp(1, 255) as u8
}

/// Frames between one mark fading and the next. A second and a half, the
/// middle of the proposal's "a second or two": stalling should clean you, and
/// a tally left alone for the length of a full cap should be gone.
pub fn mark_fade() -> u16 {
    oven::scalar(Scalar::MarkFade).max(1) as u16
}

/// What each mark adds to the swing that spends it, as a multiple of the
/// swing. Four tenths, so a full tally of five triples it.
pub fn mark_worth() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::MarkWorth))
}

/// The stagger a cash-in at a full tally applies. Half a second: a hard stop,
/// behind the hardest condition in her kit -- the rule `ability-spec.md` sets
/// for staggers.
pub fn cash_stagger() -> u16 {
    oven::scalar(Scalar::CashStagger).max(0) as u16
}

// ---------------------------------------------------------------------------
// Health, per class
// ---------------------------------------------------------------------------

/// A class's health, as `max_health` times its own multiplier.
///
/// Per class the way jump, gravity and fall already are. The Reaver starts at
/// three quarters, because a class whose whole pattern is *not being there*
/// should be the one that cannot afford to be. The Bulwark's number is the
/// Bulwark thread's to set -- its v2 wants it highest -- and everybody else is
/// one until a design says otherwise.
pub fn health_of(class: crate::class::Class) -> i32 {
    use crate::class::Class;
    let knob = match class {
        Class::Bulwark => Scalar::HealthBulwark,
        Class::Champion => Scalar::HealthChampion,
        Class::ShadowReaver => Scalar::HealthReaver,
        Class::Elementalist => Scalar::HealthElementalist,
        Class::BloodMage => Scalar::HealthBloodMage,
        Class::DualMage => Scalar::HealthDualMage,
    };
    Fx::from_int(max_health())
        .mul(Fx::from_raw(oven::scalar(knob)))
        .to_int()
        .max(1)
}

/// What a jump out of the dash's carry keeps of the dash's speed, flat.
///
/// A fifth, 2026-09-23. It used to be everything: the dash arrived at fifty
/// metres a second, slid on past the shadow, and a jump in the carry took the
/// whole slide up with it and cleared the arena. The dash stops on the shadow
/// now, and the jump out of it is a lunge -- ten metres a second, about one
/// and a half walks -- rather than a launch.
pub fn dash_jump_keep() -> Fx {
    Fx::ratio(oven::scalar(Scalar::DashJumpKeep), 100)
}

/// How fast she crosses to her shadow on a dash.
///
/// Constant while the dash runs rather than a decaying shove, so the distance
/// covered is a fact about the leash rather than about how the dodge's decay
/// happens to be tuned this week. Fast enough to cross the **whole leash**
/// inside one dodge, which is the property that makes the dash a reliable
/// escape rather than a gamble on how far away she left the thing.
///
/// That property is `reaver::the_dash_crosses_the_whole_leash`, which did not
/// exist until 2026-09-17 -- the leash went to twice the throw that day, and
/// the one thing standing between that and a dash that runs out of dodge
/// halfway across the arena was a sentence in this comment. The dash ends with
/// the dodge, so falling short means spending the dodge and arriving nowhere.
pub fn shadow_dash_speed() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::ShadowDashSpeed))
}

/// How far the Guillotine's blades travel from the shadow.
pub fn lotus_radius() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::LotusRadius))
}

/// The height above the shadow's feet that the whole flower lies in.
///
/// **The lotus is flat.** It opens in one horizontal plane, holds there and
/// closes there, so every blade is at this height for the whole of its life.
///
/// It used to arc instead: the blades left the shadow's feet, rose to a peak
/// mid-eruption and came back down to the floor at full extension. Two things
/// were wrong with that, and only one of them was how it looked. A blade at its
/// furthest reach was back at ground level, so the volume that is supposed to
/// be the punishing part of the ability spent the end of its travel half buried
/// in the floor. And a flower that changes height while it turns is hard to
/// read as a *plane* being swept, which is what a player has to judge when they
/// decide whether they are standing in one.
///
/// Midriff on a fighter `body_height` tall -- below the chest that
/// `cast_height` puts a cast at, because these come out of the shadow's waist
/// rather than its hands.
pub fn lotus_height() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::LotusHeight))
}

/// How far a blade's path bends as it goes, in turns.
///
/// Zero is twelve spokes. Anything else is what makes it a lotus: each blade
/// leaves on its own bearing and keeps turning, so the twelve of them open like
/// petals rather than a starburst.
pub fn lotus_curl() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::LotusCurl))
}

/// How far a blade turns on the way **home**, in turns, and the other way.
///
/// The return is its own spiral rather than the eruption played backwards.
/// Coming home the blade sweeps this far against the direction it opened in,
/// and it is deliberately more than `lotus_curl`, so it turns past the bearing
/// it started on instead of unwinding onto it.
///
/// **That is a hit test as much as a look.** Set equal to `lotus_curl` the
/// blade retraces its outward arm exactly -- and ground a blade has already
/// crossed is ground whose occupants have already been cut and have had the
/// whole hold to leave, so a retraced return can only catch somebody who walked
/// back into the same line. Winding past the start means the way home sweeps
/// floor the way out never touched, which is what makes the drag through a
/// crowd the ability's own description of itself rather than a second helping
/// of the first pass.
pub fn lotus_uncurl() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::LotusUncurl))
}

/// How wide a blade is, in the plane the flower lies in.
///
/// The **width** of a shuriken rather than the radius of a ball: paired with
/// `lotus_blade_thickness`, which is how thin it is across that plane. It used
/// to be both at once, at 0.45 -- wider than a fighter's own body, which is why
/// six of them read as beach balls. Twelve at 0.22 is a flower made of blades.
pub fn lotus_blade_radius() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::LotusBladeRadius))
}

/// Half a blade's thickness, across the plane it lies in.
///
/// The other half of the shuriken. A blade tested as a sphere reached from a
/// standing fighter's shins to their chest, which is not a shape anybody can do
/// anything about; a slab this thin at waist height is one they can **jump**,
/// and that is the counterplay the ability's own description always implied.
///
/// Half rather than whole because it is used either side of the plane, which is
/// where the flower actually is -- see `state::World::sliced`.
pub fn lotus_blade_thickness() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::LotusBladeThick))
}

/// Frames the blades take to reach full extension.
pub fn lotus_erupt() -> u16 {
    oven::scalar(Scalar::LotusErupt).max(1) as u16
}

/// Frames they hang there before coming back.
pub fn lotus_hold() -> u16 {
    oven::scalar(Scalar::LotusHold).max(0) as u16
}

/// Frames they spend chasing the shadow home.
pub fn lotus_return() -> u16 {
    oven::scalar(Scalar::LotusReturn).max(1) as u16
}

/// What a blade deals on the way back, as a share of what it dealt going out.
pub fn lotus_return_damage() -> Fx {
    Fx::ratio(oven::scalar(Scalar::LotusReturnDamage), 100)
}

/// How much speed a blade leaves whoever it catches.
pub fn lotus_slow() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::LotusSlow))
}

/// How far from the Elementalist a stone can be raised.
///
/// It used to put one a fixed distance straight ahead, which meant the only way
/// to place a stone anywhere was to walk there. It is a **reach** now: the stone
/// comes up where the crosshair is, and this is as far as that can be. Raised
/// from 2.5 m when the meaning changed -- a fixed 2.5 m ahead is a sensible
/// place to stand a stone, and a 2.5 m leash is barely enough room to aim in.
pub fn raise_reach() -> Fx {
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

// ---------------------------------------------------------------------------
// The Champion
// ---------------------------------------------------------------------------
//
// The class used to be three multipliers -- reach, damage and recovery, one set
// per weapon -- laid over one three-move table. Those nine numbers are gone.
// Each weapon now has its own moves with their own frame data on its own mouse
// button, which is both what the multipliers were trying to buy and something
// a multiplier cannot buy: a hammer that is *shaped* differently from a spear
// rather than a spear with bigger numbers. What is left here is Rush, which is
// the part of the class that is genuinely one mechanic rather than a move.

/// How fast the dash travels. Above the dodge, which is the point: Rush is the
/// Champion's answer to every gap in the arena, and a dash you could walk
/// alongside would not be one.
pub fn rush_speed() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::RushSpeed))
}

/// How long the dash lasts. Speed times this is the distance it covers, which
/// is the number that actually matters for spacing.
pub fn rush_frames() -> u16 {
    oven::scalar(Scalar::RushFrames) as u16
}

/// Frames after a dash ends before the charge is back.
///
/// One charge is the design (see `docs/design/champion.md`): Rush is the only
/// way out of a committed tail, so spending it has to be a decision. The
/// recharge is what makes it one.
pub fn rush_recharge() -> u16 {
    oven::scalar(Scalar::RushRecharge) as u16
}

/// How far below the horizon you have to be pointing for a spear thrown out of
/// a Rush to plant in the floor and vault instead of stabbing.
///
/// In turns, positive, measured downward. The two moves share a button and are
/// told apart by where you are looking, which is the only separator that does
/// not cost another key -- and it is also the honest one: a pole vault *is* a
/// spear aimed at the ground.
pub fn vault_pitch() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::VaultPitch))
}

/// How much of the dash's speed the vault carries into the air.
///
/// Under one: planting the spear is what turns speed into height, and a vault
/// that kept all of its run would be a jump with extra steps.
pub fn vault_carry() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::VaultCarry))
}

/// Extra upward speed from pressing jump while an uppercut has hold of
/// somebody. The "we are settling this in the air" button.
pub fn uppercut_leap() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::UppercutLeap))
}

/// Knockback multiplier against a target who is already off the ground.
///
/// Someone airborne has nothing to brace against, which is the whole reason
/// the air game is worth playing: the same swing that shoves a standing
/// fighter a metre sends a falling one across the arena.
pub fn air_hit_knockback() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::AirHitKnockback))
}

/// Damage per metre per second of impact when a spiked fighter hits the floor.
///
/// Bounded by terminal velocity rather than by this number: whatever the spike
/// sets, the fall cap is what the ground sees, so the worst case is knowable.
pub fn slam_damage() -> i32 {
    oven::scalar(Scalar::SlamDamage)
}

/// How long a spiked fighter is on the floor after landing.
pub fn slam_stagger() -> u16 {
    oven::scalar(Scalar::SlamStagger) as u16
}

/// The shove the aerial spear's fan gives the Champion when it connects.
///
/// On hit rather than on throw. It is the difference between a repositioning
/// tool and a movement option: you get the boost for *catching* somebody, so
/// the fan is thrown at people rather than at the air.
pub fn spear_fan_boost() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::SpearFanBoost))
}

/// Where a cut across the body is thrown from, as a fraction of the height
/// everything else is cast at.
///
/// **A sweep is a low cut.** The hands stay near the hip and the weapon crosses
/// the whole front, which is how the Champion's sweep has always been animated
/// -- see `crates/anim/src/clips/champion.rs`. It is a knob rather than a
/// constant because it decides what the sword can reach, and "what the sword
/// can reach" turned out to be the difference between a Ridgeback hunt with a
/// climb in it and one without: a cut thrown from the chest and dead level
/// passes over the ridge you climbed up there to break.
pub fn sweep_height() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::SweepHeight))
}

/// And how far below level that cut travels, in turns.
pub fn sweep_dip() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::SweepDip))
}

/// How far out the point of a thrust already is when the hitbox appears, as a
/// fraction of the move's reach.
///
/// Under one, because a thrust that sprang to full extension on its first
/// active frame would have no visible travel at all -- and the last of the
/// extension is what a defender is reading when they decide to step back.
pub fn thrust_extend() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::ThrustExtend))
}

// --- The ground chain ------------------------------------------------------
//
// Three hits deep, and every hit is a free choice of all three weapons. What
// these four numbers decide is the *rhythm* of that -- how long you have to
// commit to the next hit, and how much of a link's tail you serve before it can
// begin. The move table decides everything else.

/// How long a chain survives once the fighter stops swinging.
///
/// It is the window the next hit has to arrive in, and it is a real decision
/// rather than a formality: long enough to feel a beat in, short enough that a
/// string is something you commit to rather than something you can leave lying
/// around. Parked at full for as long as a link is actually running, so a slow
/// finisher cannot time its own chain out from under itself -- see
/// `state::step_mechanic`.
pub fn chain_grace() -> u16 {
    oven::scalar(Scalar::ChainGrace) as u16
}

/// How much of a connected link's recovery you serve before the next link may
/// start, when the next one is a **different** weapon. A percentage of that
/// move's own recovery, so a heavy tail still costs more than a light one.
///
/// Below [`chain_cancel_repeated`], and that gap is the whole of the class's
/// nonlinear incentive: one haft with three heads, and the weapon re-forms out
/// of the follow-through rather than being re-chambered. Set the two equal and
/// the incentive is off without anything else changing.
pub fn chain_cancel_swapped() -> u16 {
    oven::scalar(Scalar::ChainCancelSwap).clamp(0, 100) as u16
}

/// The same, for swinging the **same** weapon twice in a row.
///
/// Higher, but not by much. Repeating a weapon is meant to stay completely
/// viable -- "strong when played linearly" is a design goal for this class, not
/// a concession -- so the repeat is a few frames slower rather than a wall.
pub fn chain_cancel_repeated() -> u16 {
    oven::scalar(Scalar::ChainCancelRepeat).clamp(0, 100) as u16
}

/// How long after the jump button goes down a weapon click still takes off.
///
/// "Attack as you jump" is one intention and two buttons, and nobody presses
/// two buttons on the same frame. This is how wrong the timing may be and still
/// mean what the player meant. See `state::arm_takeoff`.
pub fn takeoff_window() -> u16 {
    oven::scalar(Scalar::TakeoffWindow) as u16
}

/// The forward shove the spear's takeoff gives, on the frame the shaft reaches
/// the floor.
///
/// The move is a jump with a weapon in it: `Move::self_lift` supplies the
/// height and this supplies the direction, so the pole drive is how a Champion
/// crosses ground and gains height in one press. Read from the live input, like
/// the aerial fan's shove -- you choose where to go as the spear lands.
pub fn pole_drive_boost() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::PoleDriveBoost))
}

/// Where the inner edge of a wing sits, as a fraction of the move's reach.
///
/// A wing is a **section of a torus** lying flat (see `moves::Shape::Wing`), and
/// this is how big the hole in it is. Near one, which is the point: the volume
/// is a **thin curved blade travelling through the air**, not a pie slice. At
/// 0.16 -- where it sat until 2026-09-13 -- the section reached from the
/// caster's own elbow out to full range on every frame, which is a filled disc
/// with a pinhole in it, and a filled disc is what this shape exists not to be.
///
/// The band it leaves runs from `wing_inner` of the reach out to the reach, so
/// this knob and `Move::reach` together say both where the blade is and how
/// deep it is. `view/tests/kinematics.rs` checks the band against the punch in
/// the baked clip, because the simulation has no idea where a fist is.
pub fn wing_inner() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::WingInner))
}

/// How far the ring's middle sits toward the caster's **other** arm, as a
/// fraction of the move's reach.
///
/// A ring centred on the body is the same distance from it at every bearing, so
/// however short the section is it reads as a piece of a halo rather than as
/// something thrown. Pushing the middle off her breaks that: the blade comes in
/// close beside her on the punching arm's side and swings wide in front, which
/// is a swipe passing by rather than a circle drawn around her.
///
/// Toward the other arm rather than the punching one, because the pivot has to
/// be on the far side of the body from the fist for the near part of the arc to
/// be the part beside that fist.
pub fn wing_offside() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::WingOffside))
}

/// And how far forward of her the ring's middle sits, as a fraction of reach.
///
/// The other half of placing the ring. It moves where the blade *finishes*
/// without changing how curved it is, which is the difference between a punch
/// that ends at arm's length and one that ends a pace in front of her.
pub fn wing_ahead() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::WingAhead))
}

/// Where the leading edge stops, in turns off straight ahead, toward the
/// punching arm's side.
///
/// **Not dead centre**, and that is the whole of it: the two autos are told
/// apart by which arm threw them, so each one has to finish in front of *its
/// own* hand. A wing that closed on the body's centre line put both of them in
/// the same place at the moment the player is reading which one landed.
///
/// Signed by the arm in `moves::wing`, so one knob mirrors.
pub fn wing_finish() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::WingFinish))
}

/// How much of a wing's span its **tip** is, on the last frame it is out.
///
/// The tip is the part that has just arrived, and it is the only part live on
/// that frame -- everything behind it has already been swept. Small, because
/// the whole point of it is that landing it is a timing decision rather than
/// something that happens to you.
pub fn wing_tip() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::WingTip))
}

/// What the tip multiplies damage by.
///
/// The one piece of execution in an attack that is otherwise thrown constantly:
/// a punch you can throw all day, with one frame in it that is worth waiting
/// for.
pub fn wing_tipper() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::WingTipper))
}

/// How big the tip is.
///
/// **The tip is a bubble, not a slice.** It is the foremost point of the ring
/// and nothing else -- one frame, at the end of the blade -- so it carries a
/// radius of its own rather than inheriting `Move::radius`, which is how deep
/// the band behind it is. Two different questions: one is how forgiving the
/// wing is about distance, the other is how forgiving the tip is about exactly
/// where the end of it was.
pub fn wing_tip_radius() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::WingTipRadius))
}

/// How far along the bar landing an auto moves the Dual mage.
///
/// The small unit. Everything else on the class is measured against it: an auto
/// is the thing you throw constantly, so it is the thing the bar is calibrated
/// in.
pub fn meter_auto_push() -> i32 {
    oven::scalar(Scalar::MeterAutoPush)
}

/// And how far a cast moves her. More than an auto, because committing to a
/// move is committing harder to a side than poking is.
pub fn meter_cast_push() -> i32 {
    oven::scalar(Scalar::MeterCastPush)
}

/// How long ascension lasts, once both bars are goaded to the top together.
///
/// **A clock rather than a state you have to escape.** The first version had no
/// exit at all: reaching the end burned you until you were nearly dead and then
/// went on burning, with nothing to do about it. The design's own answer is
/// that the drain *is* the clock (see `docs/design/dual-mage.md`).
///
/// Six seconds, from three: at three a person had time for two or three
/// abilities and it was over before they had noticed it had begun unless they
/// were watching the bar. The drain a frame halved with it, so the whole ride
/// still costs the same seventy per cent of a health bar.
pub fn ascension_frames() -> u16 {
    oven::scalar(Scalar::AscensionFrames) as u16
}

/// Health it costs per frame while it runs. It can never be the thing that
/// kills you -- the same clamp the burn already had.
pub fn ascension_drain() -> i32 {
    oven::scalar(Scalar::AscensionDrain)
}

/// The longest she is staggered when it ends: the ceiling of a stagger that
/// is **graduated** by how much of the drain her hits refunded, down to
/// [`ascension_stun_floor`]. A near miss reads as a near miss.
pub fn ascension_stun() -> u16 {
    oven::scalar(Scalar::AscensionStun) as u16
}

/// The top of either bar, in whole units. Both run from zero to this.
pub fn meter_max() -> i32 {
    oven::scalar(Scalar::MeterMax)
}

// ---------------------------------------------------------------------------
// The two bars and the hill between them -- see `crate::dual`
// ---------------------------------------------------------------------------
//
// v2 of the mechanic, 2026-09-23. The first values were chosen against the
// cadence in `docs/design/dual-mage.md`; the same day's play came back
// overtuned, and the values below are the second pass, tuned to the
// **benchmarks** in that document -- player actions and their outcomes, which
// `cargo run -p sim --bin goad` prints and `tests/dual_mage.rs` pins. The feel
// log for the date carries both passes.

/// How far apart the two bars may sit and still count as level, in whole
/// units of the bar. Inside it nothing moves on its own; outside it the hill
/// starts.
///
/// **The cadence sets it.** One cast from level has to stay inside, two have
/// to leave, the finisher always leaves, and the alternating rhythm -- an auto
/// and a cast on one side, then the same on the other -- has to stay inside.
/// So it is at least an auto plus a cast and less than two casts: with pushes
/// of 5, 9 and 20 that is between 14 and 18, and 16 is the middle of it.
pub fn meter_band() -> i32 {
    oven::scalar(Scalar::MeterBand)
}

/// How fast the higher bar rises and the lower falls, per second, for every
/// unit the gap is outside the band.
///
/// The slope of the hill. One a second per unit means a Judgement thrown from
/// level -- four outside the band -- is already at the cap, so the finisher
/// starts something the other hand has to answer within a cast or two. The
/// first pass had it at two; the runaway read as too fast.
pub fn drift_gain() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::DriftGain))
}

/// The most the drift ever moves either bar, per second.
///
/// Chosen against benchmark B6: uncorrected, a Judgement from level empties the
/// lower bar in eight to twelve seconds, and answered with the far-side hand
/// -- an auto, the cast, an auto or two -- it is back inside the band within
/// one exchange of the finisher recovering. Far-side autos alone claw it back
/// slowly: ten a second onto the low bar against a gap that widens at twice
/// this. The first pass was ten a second, and it read as bars that drained too
/// fast; the burn is what bites now, not the drift.
pub fn drift_cap() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::DriftCap))
}

/// Health per second the gap costs, per unit outside the band.
pub fn burn_gain() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::BurnGain))
}

/// The most the burn ever costs, per second.
///
/// Twenty-five, against benchmark B7: ignoring a runaway for ten seconds costs
/// about what the Judgement that started it did to them -- a fifth of a health
/// bar -- and fully one-sided for half a round costs between half and four
/// fifths. The first pass was fifteen and read as a footnote. It cannot kill
/// either way; `dual::singe_health` stops at one.
pub fn burn_cap() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::BurnCap))
}

/// How fast both bars fall on their own, per second, always.
///
/// What makes a tier something she holds by fighting. Two a second, against
/// benchmarks B4 and B5: clean alternating reaches the blink in ten to
/// fourteen seconds, the second jump in sixteen to twenty-one and the wings in
/// twenty to twenty-six -- once a round, with commitment -- and stopping at
/// three quarters keeps the blink for at least two exchanges and loses it
/// inside seven. The first pass was six, and it read as bars that drained too
/// fast; the plan's own wish for the blink to go inside one exchange was
/// dropped with it, in favour of the slower drain the person asked for.
pub fn meter_calm() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::MeterCalm))
}

/// The lower bar at which the dodge becomes a blink. Half.
pub fn tier_blink() -> i32 {
    oven::scalar(Scalar::TierBlink)
}

/// The lower bar at which she gets a second jump and a slower fall. Three
/// quarters.
pub fn tier_jump() -> i32 {
    oven::scalar(Scalar::TierJump)
}

/// The lower bar at which both are counted as full and she ascends.
///
/// A hair under the top rather than the top itself, and it has to be: the
/// calm runs every frame and the two bars are pushed by two different presses,
/// so the moment one is topped up the other has already lost a little, and
/// both can never be at exactly the top on the same frame. Ninety-five is what
/// the other bar keeps through one move's worth of calm with room to spare.
pub fn tier_wings() -> i32 {
    oven::scalar(Scalar::TierWings)
}

/// The second jump's impulse, as a share of the first.
pub fn second_jump() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::SecondJump))
}

/// What the fall cap is multiplied by while she holds the second tier.
pub fn slow_fall() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::SlowFall))
}

/// Health returned for every hit landed while ascending.
///
/// The refund the original design wrote and never built. Forty against a drain
/// of two a frame means the cost is paid back in full by eighteen hits in six
/// seconds, which nobody will do -- it is meant to keep her alive one
/// connection at a time, not to make the ride free.
pub fn ascension_refund() -> i32 {
    oven::scalar(Scalar::AscensionRefund)
}

/// The shortest the stagger on the way out gets, for a ride that paid itself
/// back. [`ascension_stun`] is the longest, for one that landed nothing.
pub fn ascension_stun_floor() -> u16 {
    oven::scalar(Scalar::AscensionStunFloor) as u16
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
///
/// Moved to 0.44 on 2026-09-17 and put back the next day, with the rise. See
/// [`structure_rise`] for why neither of them is a knob to turn on a
/// calculation.
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
// The Elementalist's auto, aimed through terrain
// ---------------------------------------------------------------------------
//
// Bolt is a beam: a ray along the crosshair, and whatever it meets first is
// the move. See `docs/design/kits/elementalist.md` and `crate::bolt`.
//
// **The beam's own length and thickness are not here.** They are the move
// table's `reach` and `radius`, because the beam *is* the move -- a second
// copy of its range would only be a number the frame table could disagree
// with. What is here is everything the beam's three outcomes need.

/// How fast a kicked stone leaves. The ceiling `stones::launch_decel` eases
/// down from over the back of its travel.
pub fn bolt_knock_speed() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::BoltKnockSpeed))
}

/// How far a kicked stone travels before it is considered spent. Measured
/// from where the kick began, not a lifetime -- a stone parked by a wall
/// partway through still counts the same distance it would have covered.
pub fn bolt_knock_range() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::BoltKnockRange))
}

/// Where in that travel the stone starts dying off, as a fraction of the
/// whole. Below it the stone keeps the shot's full speed; above it, speed
/// eases toward zero by the time the travel runs out.
pub fn bolt_knock_decel_start() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::BoltKnockDecelStart))
}

/// Below this closing speed a kicked stone is too slow to call a hit. It
/// still shoves like any other stone; it just does not hurt anyone.
pub fn bolt_knock_min_speed() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::BoltKnockMinSpeed))
}

/// Damage per whole metre-per-second of closing speed, so a stone kicked at
/// someone close to the point-blank ceiling hits far harder than one that has
/// mostly died off by the time it reaches them.
pub fn bolt_knock_damage_per_speed() -> i32 {
    oven::scalar(Scalar::BoltKnockDamagePerSpeed)
}

pub fn bolt_knock_stagger() -> u16 {
    oven::scalar(Scalar::BoltKnockStagger) as u16
}

/// How much of the closing speed a caught fighter is pushed with.
pub fn bolt_knock_push() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::BoltKnockPush))
}

// The fire bolt: what leaves a pillar the beam was aimed through. The one
// thing she throws that has a speed, and the only part of the auto that can
// be dodged after it is thrown.

/// How fast it flies. Fast enough that it is a poke rather than a lob -- it
/// crosses the arena in about a second and is not meaningfully led.
pub fn fire_bolt_speed() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::FireBoltSpeed))
}

/// How far it gets before it is spent. A distance rather than a lifetime, so
/// retuning the speed does not silently retune the range with it.
pub fn fire_bolt_range() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::FireBoltRange))
}

/// How thick it is. Small: a bolt you can walk out of the way of.
pub fn fire_bolt_radius() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::FireBoltRadius))
}

/// Low to middling -- more than the beam's own poke, far less than anything
/// she commits frames to.
pub fn fire_bolt_damage() -> i32 {
    oven::scalar(Scalar::FireBoltDamage)
}

/// A little stagger, unlike the beam, which has none. The bolt is the part of
/// the auto that had to be earned by putting a pillar down first.
pub fn fire_bolt_stagger() -> u16 {
    oven::scalar(Scalar::FireBoltStagger) as u16
}

pub fn fire_bolt_blockstun() -> u16 {
    oven::scalar(Scalar::FireBoltBlockstun) as u16
}

pub fn fire_bolt_knockback() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::FireBoltKnockback))
}

// Cataclysm: the heavy, on right click. See `crate::effects::EffectKind::FireTornado`
// for what a fire pillar becomes, and `crate::debris` for what a structure
// scatters into. Neither invents a hitbox of its own where an existing one
// already answers the question -- the tornado ticks and is stood in exactly
// like the pillar it was, and the debris is thrown along the beam's own line.

/// How fast a loosed pillar crosses the arena. Visibly faster than a walk, so
/// outrunning one head-on is not an option -- stepping off its line is.
pub fn tornado_speed() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::TornadoSpeed))
}

/// Acceleration toward the tornado's own live centre for anyone caught inside
/// its pillar volumes, in the same units gravity is -- comparable in
/// strength, deliberately, so getting pulled in reads as a real force rather
/// than a nudge.
pub fn tornado_pull() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::TornadoPull))
}

/// How long a loosed pillar can fly before it burns out on its own clock,
/// counted from the frame it was cut loose rather than from when it was
/// first planted -- see `Effect::tornado_pos` and `state::World::step_effects`.
/// Separate from how grown it is: growth is `pillar_life`'s question, answered
/// continuously from the moment it was planted, so a pillar cut loose late in
/// its life is already mature and one cut loose early keeps growing into
/// itself as it travels -- either way, this is only asking how much longer it
/// gets to exist once it starts moving.
pub fn tornado_travel_life() -> u16 {
    oven::scalar(Scalar::TornadoTravelLife) as u16
}

/// How fast each piece of a broken structure flies.
pub fn debris_speed() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::DebrisSpeed))
}

/// How far a piece travels before it burns out unspent -- a shotgun has a
/// falloff range, not an infinite one.
pub fn debris_range() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::DebrisRange))
}

/// How fat a piece is for its own hit test.
pub fn debris_radius() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::DebrisRadius))
}

/// The cone's half-angle, in turns, from the beam's own line to the ring of
/// outer pieces -- see `crate::debris::blast`. The shotgun's spread, not its
/// damage falloff -- there is none of that, a piece hits for the same either
/// way.
pub fn debris_spread() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::DebrisSpread))
}

/// Damage a single piece deals -- this, times however many pieces actually
/// connect, is what a close-range blast is worth over a far one.
pub fn debris_damage() -> i32 {
    oven::scalar(Scalar::DebrisDamage)
}

pub fn debris_stagger() -> u16 {
    oven::scalar(Scalar::DebrisStagger) as u16
}

pub fn debris_blockstun() -> u16 {
    oven::scalar(Scalar::DebrisBlockstun) as u16
}

pub fn debris_knockback() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::DebrisKnockback))
}

// ---------------------------------------------------------------------------
// The Blood mage
//
// Everything she throws costs blood and gives it back on the hit, and those two
// numbers are per move -- they are in the move table beside the damage, where
// the rest of an ability's economy lives. What is here is the *shape* of the
// three things she puts into the world: how tall the spike stands, how far the
// blade flies, and how wide the arms of a Grasp open before they close.
// ---------------------------------------------------------------------------

/// How tall the black spike stands out of the ground.
///
/// Cosmetic and gameplay at once, and the reason it is a number rather than a
/// constant in the renderer: the spike is the tell. A field that is only a
/// stain on the floor is one you do not see until you are standing in it, which
/// makes a placement ability into a trap, and the class wants you to look at
/// the thing and decide to walk around it.
pub fn spike_height() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::SpikeHeight))
}

/// Frames the Bloodletter takes to fly out and come all the way home.
///
/// The whole flight, both passes. Half of it is the way out.
pub fn bloodletter_flight() -> u16 {
    oven::scalar(Scalar::BloodletterFlight) as u16
}

/// How fat the blade is. Small: it is a thrown knife, not a wave.
pub fn bloodletter_radius() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::BloodletterRadius))
}

/// Frames the Grasp's four arms take to open and converge.
pub fn grasp_flight() -> u16 {
    oven::scalar(Scalar::GraspFlight) as u16
}

/// How far off the centre line the arms bow at their widest.
///
/// The reason the ability is not just four copies of one skillshot: they leave
/// as a cone and arrive as a point, so the volume they sweep is a lens rather
/// than a line, and standing anywhere inside it gets you clipped by one or two.
/// All four is a much smaller place to be, which is what makes the root a read.
pub fn grasp_spread() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::GraspSpread))
}

/// How fat one arm is.
pub fn grasp_arm_radius() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::GraspArmRadius))
}

/// Frames the arms hold somebody still before they start pulling.
///
/// The front of the hold rather than a separate state: you are caught, and for
/// a moment nothing else happens. It is what stops the haul reading as a
/// teleport, and it is the window the caster spends starting whatever is
/// supposed to meet them.
pub fn grasp_bind() -> u16 {
    oven::scalar(Scalar::GraspBind) as u16
}

/// How big the aim marker is drawn. Presentation, and a knob because a marker
/// you cannot pick out of the arena is the same as no marker.
pub fn grasp_mark() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::GraspMark))
}

/// How fast anything that grabs hauls its catch in, in metres per second.
///
/// Fast, and finite. A blink reads as the game moving somebody for you; a haul
/// you can watch is a thing that happened to them, and it is the difference
/// between the ability landing and the ability *looking* like it landed. One
/// knob for every grab in the game, because it is a property of being dragged
/// rather than of the thing doing the dragging.
///
/// **Raised with the Grasp's reach on 2026-09-17**, and `feel.rs` is what
/// noticed: the hold is a fixed number of frames and the haul has to cover the
/// whole reach inside it, so lengthening the throw without this drops a
/// full-range catch halfway home -- standing in mid-air, mid-drag, suddenly
/// able to walk. Invisible at short range and total at long range, which is the
/// worst way for a number to be wrong.
pub fn reel_speed() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::ReelSpeed))
}

/// What a Blood mage's damage is multiplied by against something that cannot
/// move -- rooted, staggered, held, or a creature on its side.
///
/// The class's damage identity, and the reason its root is a setup rather than
/// a small reward. See `class::Class::preys_on_the_disabled` for which classes
/// it applies to and `state::Player::disabled` for what counts.
pub fn disabled_damage_mul() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::DisabledDamageMul))
}

/// How far below the horizon a melee swing stays level.
///
/// The camera sits above the shoulder, so looking *at* someone standing at your
/// own height means looking slightly down at them. Without a dead zone a swing
/// that followed the camera would tilt into the floor every time you fought
/// anybody. Inside it the swing is the standard arc in front of the body;
/// outside it, up or down by whatever is left over. Degrees, because that is
/// how the rest of the camera's angles are written. See `crate::aim::swing_path`.
pub fn swing_level_to() -> i32 {
    oven::scalar(Scalar::SwingLevelTo)
}

// ---------------------------------------------------------------------------
// The Ridgeback
//
// See `docs/design/monsters.md`. Five groups, edited separately because they
// answer different questions: how the animal is built, how it decides, what its
// hide is worth, what it takes to move it around, and what it takes to stay on
// its back.
// ---------------------------------------------------------------------------

/// Multiplies every bone offset and every part box. The one number you actually
/// reach for while playing -- *how big is it* -- which is why the proportions
/// in `beast::REST` and `beast::SHAPES` are constants and this is not.
///
/// At one the creature is about thirteen metres nose to tail with its back
/// five and a half metres up. That height is chosen against the jump: the
/// lowest thing on the standing animal is out of every class's hop but the
/// Dual mage's, and the ways up are the ones the ground game earns. **The
/// climb is a positioning problem before it is a timing one**, and this is
/// the number that decides it -- though the shape itself is `beast::REST`,
/// and the height is in the legs. See `cargo run -p sim --bin beastcheck`.
pub fn monster_scale() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::MonsterScale))
}

/// Pool health. The per-part vulnerabilities below are what decide how long a
/// fight actually takes, so this is the *second* number to reach for.
pub fn monster_health() -> i32 {
    oven::scalar(Scalar::MonsterHealth)
}

/// Health of one foot. Breaking a foot drops that corner of the animal for
/// good and puts it on its knee for a moment, which is the ground game's whole
/// payout -- so this is the number that says how long the ground game is.
pub fn limb_health() -> i32 {
    oven::scalar(Scalar::LimbHealth)
}

/// How far from the arena wall it is kept. Thirteen metres of animal in a
/// twenty-eight metre arena needs somewhere to stand.
pub fn monster_margin() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::MonsterMargin))
}

/// Walking, backing off, and how quickly it changes between them. The walk is
/// below the player's own on purpose: inside striking distance it catches you
/// by cornering you, not by outrunning you. The walk is also where the gait
/// has fully changed over from standing; above it, it blends to the gallop.
pub fn monster_walk() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::MonsterWalk))
}
pub fn monster_back() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::MonsterBack))
}
pub fn monster_accel() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::MonsterAccel))
}

/// The speed the gait has fully changed over to a gallop at, **and the fastest
/// it goes**: wanting to close a long gap runs it up to this. Above the
/// player's walk on purpose, since 2026-09-23 -- a fight you could walk away
/// from at leisure was not a hunt. Between this and the walk the two cycles
/// are blended, which is what stops a creature accelerating out of a walk from
/// planting two feet at once.
pub fn gallop_speed() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::GallopSpeed))
}

/// The distance it tries to hold, and how hard it corrects toward it. Set near
/// the middle of the move set's range band so that most of what it wants to do
/// is available most of the time. The correction is scaled by how squarely it
/// faces the target -- it turns before it runs -- see `Monster::walk`.
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
/// What each broken leg does to the turn toward that side. Applied once per
/// break, so an animal with both legs gone on one side barely comes round at
/// all -- which is a thing the player did, and can see.
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

/// Damage to a weak point fills the poise pool; a full pool is a topple.
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

/// Down on a knee: the middle rung of the ladder, and the one the ground game
/// aims at. Long enough to cross the ground and climb on -- that is what the
/// number is for, so it is set against the run-up rather than against a feeling.
pub fn stumble_frames() -> u16 {
    oven::scalar(Scalar::StumbleFrames) as u16
}

/// The cheap reaction. It interrupts a startup or a recovery but never an
/// active frame, and only a hit that hurts causes one -- which in practice
/// means a committed move rather than an auto.
pub fn flinch_frames() -> u16 {
    oven::scalar(Scalar::FlinchFrames) as u16
}
pub fn flinch_threshold() -> i32 {
    oven::scalar(Scalar::FlinchThreshold)
}

/// The hide. A damage multiplier per part: below one is armour, above one is a
/// weak point, and the spread between them is the difference between a
/// twenty-second fight and a two-minute one.
///
/// Three of these are the design in miniature. The **nape** is the highest
/// number on the animal and is reachable only from its own shoulders. The
/// **ridge** is the other weak point and is what a rider walks to first. The
/// **foot** is the softest thing a fighter standing on the floor can reach at
/// all, which is what makes the ground game a route up rather than a chore.
pub fn vuln_head() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::VulnHead))
}
pub fn vuln_neck() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::VulnNeck))
}
pub fn vuln_nape() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::VulnNape))
}
pub fn vuln_shoulder() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::VulnShoulder))
}
pub fn vuln_barrel() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::VulnBarrel))
}
pub fn vuln_ridge() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::VulnRidge))
}
pub fn vuln_haunch() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::VulnHaunch))
}
pub fn vuln_tail() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::VulnTail))
}
pub fn vuln_tail_mid() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::VulnTailMid))
}
pub fn vuln_tail_tip() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::VulnTailTip))
}
pub fn vuln_leg() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::VulnLeg))
}
pub fn vuln_foot() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::VulnFoot))
}

/// What a broken leg leaves behind, once the stumble is over.
///
/// The corner drops, the body pitches toward the missing end, rolls toward the
/// missing side, and the useless limb folds rather than punching through the
/// floor. The drop is the one that matters for the fight: enough of them and
/// the back comes within reach of a standing jump.
pub fn leg_drop() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::LegDrop))
}
pub fn leg_pitch() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::LegPitch))
}
pub fn leg_roll() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::LegRoll))
}
pub fn leg_fold() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::LegFold))
}
/// How much of the fold the joint *above* the break takes. A limb that folds
/// only at its lower joint sticks out sideways; one that folds at both tucks
/// under the body, which is what a broken leg does.
pub fn leg_buckle() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::LegBuckle))
}
/// Speed lost per broken leg, as a multiplier applied once per break.
pub fn leg_speed_hurt() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::LegSpeedHurt))
}

/// **The crowd-control model.** See `docs/design/monsters.md` §4.
///
/// `strain` is damage taken recently: every hit adds to it and
/// `strain_decay` per cent of it bleeds away every frame, so a burst fills it
/// and a trickle does not. Above `cc_strain` the creature is susceptible to
/// control at all; above `interrupt_strain` a hit breaks it out of whatever it
/// is doing, live hitbox included.
///
/// `strain_desperation` is how far both bars fall by the time it is nearly
/// dead. That one number is the arc of a hunt: methodical while the animal is
/// fresh, frantic once it is not.
pub fn strain_decay() -> i32 {
    oven::scalar(Scalar::StrainDecay)
}
pub fn cc_strain() -> i32 {
    oven::scalar(Scalar::CcStrain)
}
pub fn interrupt_strain() -> i32 {
    oven::scalar(Scalar::InterruptStrain)
}
pub fn strain_desperation() -> i32 {
    oven::scalar(Scalar::StrainDesperation)
}

/// How much of a slow it actually feels, once it is susceptible. Zero means a
/// slow never touches it; one means it takes the same slow a fighter does.
pub fn cc_slow_bite() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::CcSlowBite))
}
/// Frames rooted per frame the grab would have held a fighter.
pub fn cc_root() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::CcRoot))
}
/// Frames of stumble per metre per second of launch. A knock-up on something
/// this heavy is a trip rather than a lift, and this is the exchange rate.
pub fn cc_stumble() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::CcStumble))
}

/// How violent the shake is, as a multiplier on the baked clip.
///
/// The one animation number that is genuinely tuning rather than content: it
/// decides whether a braced rider stays on, and that is a feel question you
/// answer by playing. Everything else about the creature's motion lives in the
/// clips -- see `crates/anim/src/beast/`.
pub fn shake_force() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::ShakeForce))
}

/// Metres per cycle of the gait, and how fast it breathes when standing.
///
/// The gait is indexed by ground covered rather than by time, so this is what
/// decides where a footfall lands. The same rule the fighters' locomotion
/// follows, for the same reason: a cycle on a fixed cadence skates.
pub fn gait_stride() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::GaitStride))
}
pub fn breath_rate() -> u16 {
    oven::scalar(Scalar::BreathRate) as u16
}

/// How far round the neck will turn to keep you in view, in turns. Split across
/// three bones, so it curves rather than hinging.
pub fn head_track() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::HeadTrack))
}

// ---------------------------------------------------------------------------
// Riding
// ---------------------------------------------------------------------------

/// How close a falling body has to be to a mountable face to land on it.
pub fn mount_snap() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::MountSnap))
}

/// How far you may overhang the edge of a part before you are off it, as a
/// fraction of your own width.
pub fn edge_grace() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::EdgeGrace))
}

/// **The buck.** Acceleration of the surface under your feet, above which you
/// come off it. Not a flag on a move: a move that whips the tail throws whoever
/// is on the tail and does nothing to somebody on the shoulder, and nobody had
/// to write that down.
pub fn grip() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::Grip))
}
/// Crouching multiplies grip. The answer to a shake that is neither "dodge" nor
/// "leave", and a real decision because bracing costs you the attack you were
/// about to throw.
pub fn brace_grip() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::BraceGrip))
}

/// What being thrown off does to you.
pub fn throw_kick() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::ThrowKick))
}
pub fn throw_lift() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::ThrowLift))
}
pub fn throw_stun() -> u16 {
    oven::scalar(Scalar::ThrowStun) as u16
}

/// **What a buck costs you.**
///
/// Nothing the creature throws can reach its own back, so a rider is safe from
/// every hitbox it has -- which would make the back a room you sit in if losing
/// your grip were free. It is not: you came off four and a half metres of
/// animal, helpless, and you land.
///
/// It is also what makes the two answers to a buck worth knowing. Bracing and
/// jumping both cost something -- the attack you were about to throw, or the
/// read you have to get right -- and neither is worth paying for unless the
/// alternative has a price.
pub fn throw_damage() -> i32 {
    oven::scalar(Scalar::ThrowDamage)
}

/// Frames after landing before the grip test starts. You get a moment to plant
/// your feet; without it the first frame aboard looks like an enormous
/// acceleration, because it is.
pub fn mount_settle() -> u16 {
    oven::scalar(Scalar::MountSettle) as u16
}

/// Walking speed on the creature's back, as a fraction of the ordinary one.
pub fn rider_speed() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::RiderSpeed))
}

/// **The cap on what a jump off the creature carries with it.**
///
/// A leap takes the surface's own velocity, which is what makes stepping off
/// the back of a charging animal a real option. Uncapped, it also makes
/// *jumping the shake* impossible: the back is whipping sideways at forty
/// metres a second on the frame you leave it, so the answer to a buck would be
/// to get flung slightly further away by choice.
///
/// You can only take with you as much momentum as you had time to push off
/// against, so the carry is capped. That single line is what turns the shake
/// from a thing you brace through into a thing you can read and hop -- and
/// getting the timing wrong still throws you, because a late jump is a jump
/// that never left the ground. See `docs/design/monsters.md` §3.
pub fn leap_carry() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::LeapCarry))
}

/// Where the creature stands when a hunt begins, and where the hunters do.
pub fn monster_spawn() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::MonsterSpawn))
}
pub fn hunter_spawn() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::HunterSpawn))
}

/// A step you can walk up rather than having to jump.
///
/// The creature is terrain, not a set of ledges: a rider who walks into a
/// mountable face this close above their feet is put on top of it instead of
/// stopped by it, which is what every platformer does and what lets a climb
/// that starts at the tail reach the ridge.
pub fn step_up() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::StepUp))
}

// ---------------------------------------------------------------------------
// The Elementalist, off the ground
// ---------------------------------------------------------------------------
//
// Three moves, and everything that is not a frame count is here. What *is* a
// frame count -- startup, active, recovery, the hang -- stays in the move
// table, along with the two things the shots take from their own row: how far
// they reach and how thick they are. A second copy of either would only be a
// number the frame table could disagree with, which is the same argument the
// beam's own range makes above.
//
// See `docs/design/kits/elementalist.md` and `crate::gust`.

/// How fast the Air bolt travels.
///
/// Slower than a fire bolt on purpose: the fire bolt is the payoff for putting
/// a pillar between you and somebody, and this is the shot she throws
/// constantly. A poke you can see coming is a poke you can answer, which is
/// what makes throwing it a decision rather than a reflex.
pub fn air_bolt_speed() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::AirBoltSpeed))
}

/// How fast the Gale disc travels.
///
/// Half the Air bolt's, and the slowest thing she throws. It has to be walked
/// away from: a disc that grows into a real hit at the far end of its travel
/// is only a decision if the target has time to decide.
pub fn gale_speed() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::GaleSpeed))
}

/// How big the Gale is as it leaves her hand, as a share of the size it
/// reaches at the end of its travel.
///
/// The move table's `radius` is the far end; this is the near one. What it
/// buys is the whole of the move's spacing: damage and knockback ride the same
/// fraction, so a disc caught at point-blank range is a puff of air and one
/// caught at the tip is the heaviest shove in the kit. The same inversion
/// Flame spitter is written around -- see the kit -- and the opposite of every
/// other projectile in the game.
pub fn gale_start() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::GaleStart))
}

/// How far the Gale has to travel to be all of itself.
///
/// **Not its reach, and that is the point.** How far a shot goes and how
/// quickly it comes up to size are two decisions, and one number for both means
/// every change to the range silently retunes the growth: double the reach and
/// the disc is half as big everywhere a fighter actually stands. Past this it
/// simply stays full size for the rest of its flight, which is the shape a
/// thrown disc has anyway -- it opens, and then it is open.
pub fn gale_grow() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::GaleGrow))
}

/// How fast Landfall drives her at the floor once the wind-up is spent.
///
/// Fast enough that the plunge reads as a commitment rather than a fall, and
/// slow enough that it is a *descent* somebody can hit her out of -- which is
/// most of what the move is. From the top of her jump it lasts about as long
/// again as the hang did.
pub fn landfall_dive() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::LandfallDive))
}

/// How far in front of her the Landfall stone comes up.
///
/// A distance rather than a reach, and the only placement in the class the
/// crosshair does not decide: she is arriving, not aiming. Just past a body
/// radius plus a stone's, so the slab stands clear of where she lands rather
/// than shoving her off her own arrival. See `aim::planted_ahead`.
pub fn landfall_ahead() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::LandfallAhead))
}

/// How long the Landfall stone takes to come out of the floor.
///
/// Well under `structure_rise`, and that is the point: an ordinary stone is
/// raised and the rise is its telegraph, while this one is *driven* up by a
/// body hitting the ground and has already been telegraphed by the plunge
/// that put it there. The two phases still come off the same curve, so the
/// churn and the eruption move with it -- see `crate::stones`.
pub fn landfall_rise() -> u16 {
    oven::scalar(Scalar::LandfallRise).max(1) as u16
}

/// How far above the floor the Landfall stone throws what it erupts under,
/// in turns.
///
/// An eighth of a turn is forty-five degrees: up and away in equal measure. A
/// slab levered out of the ground at an angle throws you along the angle, so
/// this is a push *and* a pop rather than either on its own -- and pointing it
/// away from her is what makes the move a way of clearing the space she has
/// just landed in.
pub fn landfall_tilt() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::LandfallTilt))
}

/// How hard the Landfall stone's eruption throws whoever is standing over it.
///
/// An ordinary eruption does damage and a stagger and leaves you where you
/// were. This one adds the shove, along the angle above. The damage and the
/// stagger are still `stone_erupt_damage` and `stone_erupt_stagger` -- it is
/// the same eruption, leaning.
pub fn landfall_erupt() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::LandfallErupt))
}

// --- The step, the diagonal, and going up with them ------------------------

/// How many frames before the hitbox a stepping move starts driving the body.
///
/// Shared across the whole roster, because the **shape** of a step is a rule and
/// only its size is a per-move decision: every stepping move drives for this many
/// frames and then the frame its hitbox appears on, so the feet always arrive
/// with the weapon rather than some frames before or after it. A long distance
/// over that fixed window is a dash and a short one is a step, which is why
/// [`crate::moves::Move::step`] is one number rather than two.
///
/// The run-up is what stops a step being a teleport on the frame of contact: the
/// body is already travelling when the weapon lands, which is the whole of what
/// "the body goes with it" looks like from the other side.
pub fn step_lead() -> u16 {
    oven::scalar(Scalar::StepLead) as u16
}

/// How far a diagonal cut's plane is rolled off the vertical, in turns.
///
/// Zero is [`crate::moves::Plane::Upright`] and a quarter turn is
/// [`crate::moves::Plane::Flat`], so this slider's two ends are the other two
/// planes and everything interesting is in the middle. An eighth of a turn is
/// corner to corner: the head starts as far to the side as it does above, which
/// is what makes the cut own the width of the front *and* the height of a body
/// instead of choosing.
///
/// **One magnitude, two mirrored cuts.** The sign lives in
/// [`crate::moves::Plane::Diagonal`]'s own hand, the same way the Dual mage's
/// two wings share one span -- so a tuner cannot roll the sword's first cut and
/// forget its second.
pub fn cut_roll() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::CutRoll))
}

/// What the hammer finisher's knock-up is multiplied by when the Champion jumps
/// into it.
///
/// Above one, and the whole of what the decision buys: the finisher throws them
/// up either way, and electing to go with them throws them *higher* as well as
/// putting you there to meet them. See `state::going_up_with_them`.
pub fn hammer_leap_launch() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::HammerLeapLaunch))
}

/// The Champion's own upward speed on the frame that finisher connects, when the
/// jump was pressed during it.
///
/// Paid **on contact**, not on the press, which is the same rule the aerial
/// fan's shove follows and for the same reason: a whiffed finisher that still
/// launched you into the air would be a free escape attached to the most
/// punishable move in the kit. You go up because it landed.
pub fn hammer_leap() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::HammerLeap))
}

// ---------------------------------------------------------------------------
// The Dual mage's depth curve
// ---------------------------------------------------------------------------
//
// The class's founding idea: **power scales continuously with the bar of the
// force a move is made of.** Two numbers describe the whole of it, and one
// function reads them -- `state::depth`, through `dual::depth_at` -- so that "a
// cast from a full bar is a bigger cast" is one rule rather than a thing each
// ability remembers to do. See `docs/design/dual-mage.md`.

/// What a cast from an empty bar is worth, as a multiplier.
///
/// **Below one, and it has to be.** A bar she has not goaded is a being she has
/// not fed, and the only way to say that in numbers is to make an empty bar
/// cost her something. Everything she throws from empty comes out thin.
///
/// Six tenths, and the ceiling is fourteen tenths: the first pass was half to
/// double, and a Judgement from a full bar was forty per cent of a health bar.
/// The spread is narrower now, and the base damage came down with it -- see
/// benchmarks B1 to B3 in `docs/design/dual-mage.md`.
pub fn depth_floor() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::DepthFloor))
}

/// And what the same cast is worth from a full bar.
///
/// Above one, by as much as a full bar is meant to be frightening. The gap
/// between this and [`depth_floor`] is the reason to goad; the hill between
/// the two bars is the reason not to goad one of them alone.
pub fn depth_ceiling() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::DepthCeiling))
}

/// How much of the depth curve the **size** of a volume takes, as a percentage.
///
/// Damage always rides the whole curve. Whether a deep cast is also *bigger* is
/// a separate question with a separate answer, because the two are different
/// kinds of threat: more damage is worse to be hit by, more radius is harder to
/// not be hit by. A hundred is the two moving together.
///
/// The two autos never grow whatever this says -- see
/// `moves::dual::is_an_auto`, which is where that exemption is declared and
/// why.
pub fn depth_size() -> Fx {
    Fx::ratio(oven::scalar(Scalar::DepthSize), 100)
}

/// How far along the bar throwing the finisher moves her.
///
/// The third tier, and the one `docs/design/dual-mage.md` has been asking for
/// since the bar was built: "a deep finisher nearly throws you over the edge".
/// An auto is the small unit, a cast is more, and this is the one that makes
/// casting Judgement from depth a question about whether you survive it rather
/// than a free payoff.
pub fn meter_finisher_push() -> i32 {
    oven::scalar(Scalar::MeterFinisherPush)
}

/// What landing the wing's **tip** multiplies the shove by.
///
/// The tip already hits harder ([`wing_tipper`]); this is the other half of
/// the same execution, and it is what makes which arm you punch with a
/// *spacing* decision. The light auto's real knockback lives out at the far
/// edge of the blade, so a player who wants somebody moved has to stand at the
/// end of their own range to do it -- and on the dark arm the same frame pulls
/// them that much further in instead, because the sign of the knockback is what
/// says which. See `moves::Move::knockback`.
pub fn wing_tip_shove() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::WingTipShove))
}

/// What Sweep's **dark** form leaves on whoever it caught: their walking speed,
/// as a multiplier.
///
/// Sweep is one move with two answers rather than two moves, because the shape
/// is the same either way -- both arms across the whole front, driven from the
/// hips. What changes is what happens to the people it caught: light throws
/// them off their feet, dark takes their legs out from under them in the other
/// sense. The light form spends the move's own `knockback` and `launch`; the
/// dark form spends this and [`sweep_heal`].
pub fn sweep_slow() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::SweepSlow))
}

/// And what the dark form gives her back, per target caught.
///
/// Per target rather than a share of the damage, which is the point of it being
/// the panic button: a Sweep that catches two people is worth twice as much as
/// one that catches one, so the answer to being swarmed is the same move as the
/// answer to being cornered.
pub fn sweep_heal() -> i32 {
    oven::scalar(Scalar::SweepHeal)
}

/// How wide the light Lance's burst is where the line ran out.
pub fn lance_burst_radius() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::LanceBurstRadius))
}

/// What that burst deals.
///
/// More than the line that carried it, and deliberately: the line is the
/// delivery and the burst is the ability. That is what makes it a thing you aim
/// *past* somebody -- the far end of the throw is where the damage is, so
/// landing it means picking a point behind them rather than on them.
pub fn lance_burst_damage() -> i32 {
    oven::scalar(Scalar::LanceBurstDamage)
}

/// And how long it hangs there.
pub fn lance_burst_life() -> u16 {
    oven::scalar(Scalar::LanceBurstLife) as u16
}

/// How long the dark Lance's tether can hold, at most.
pub fn tether_life() -> u16 {
    oven::scalar(Scalar::TetherLife) as u16
}

/// What it takes out of whatever it caught, per tick.
pub fn tether_drain() -> i32 {
    oven::scalar(Scalar::TetherDrain)
}

/// How far the two of them may get apart before the line parts.
///
/// **The whole cost of the ability.** A drain that held at any range would be a
/// ranged class's tool on a melee class's kit; breaking on distance is what
/// keeps her standing next to the thing she is draining, which is exactly where
/// a fragile body does not want to be. It is the same argument as autoing back
/// toward centre, made by a different ability.
pub fn tether_leash() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::TetherLeash))
}

/// How wide the field Judgement leaves is.
pub fn judgement_field_radius() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::JudgementFieldRadius))
}

/// What that field deals per tick. Low: the strike is the damage, and the field
/// is the ground it leaves behind.
pub fn judgement_field_damage() -> i32 {
    oven::scalar(Scalar::JudgementFieldDamage)
}

/// And how long it lasts before the depth curve gets hold of it.
pub fn judgement_field_life() -> u16 {
    oven::scalar(Scalar::JudgementFieldLife) as u16
}

/// How fast she moves while standing in her own field.
///
/// The kit's "moving through it grants you speed", which is the half of the
/// field that is for her rather than against them. Above one: the field is a
/// place she wants to be, so that a Judgement thrown at somebody's feet is also
/// a Judgement thrown at her own next few seconds.
pub fn judgement_field_speed() -> Fx {
    Fx::from_raw(oven::scalar(Scalar::JudgementFieldSpeed))
}
