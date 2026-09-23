//! The Oven: every tuned number in the game, live.
//!
//! Feel work is a loop — change a number, play it, change it again — and the
//! loop is only as fast as its slowest step. With the values compiled in, that
//! step is a rebuild, so in practice you change one number, wait, and lose the
//! comparison you were trying to make. The Oven makes the numbers runtime state
//! so the loop closes in a frame, and adds a **bake** that writes them back to
//! the repository so a session of tuning ends as a commit rather than as
//! something you have to remember and retype.
//!
//! ## Every knob is an `i32`
//!
//! Fixed-point values are stored as their raw 16.16 bits, frame counts as
//! frames, health as health. One representation means one store, one editor
//! widget and one file format for three hundred parameters that would otherwise
//! each need their own. `Unit` says how to read the integer back.
//!
//! ## Determinism
//!
//! These are *rules*, not state: they never change during a frame, so rollback
//! neither saves nor restores them. But two peers running different rules would
//! desync silently and look like a netcode bug, so the tuning hash is folded
//! into `World::checksum` — a mismatched Oven shows up immediately as a desync
//! rather than as a mystery.

use crate::class::{ALL_CLASSES, Class};
use crate::fixed::Fx;
use std::sync::LazyLock;
use std::sync::atomic::{AtomicI32, Ordering};

use crate::tuned;

/// How to read a knob's integer back, and how to show it to a person.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Unit {
    /// Raw 16.16 fixed point.
    Fixed,
    /// Whole frames at 60 Hz.
    Frames,
    /// A plain count.
    Int,
    /// 0–100.
    Percent,
    /// 0 or 1.
    Flag,
}

/// Raw helper for writing fixed-point bounds readably.
const fn fx(num: i32, den: i32) -> i32 {
    Fx::ratio(num, den).raw()
}

impl Unit {
    /// Render a raw value for a human, using integer arithmetic only.
    ///
    /// No `f32` anywhere in this crate, Oven included: `sim` is float-free as a
    /// determinism guarantee, and an editor is not a good enough reason to put
    /// the first one in. Converting for display is the palette's job; this
    /// exists so a baked file can carry a readable comment.
    pub fn show(self, raw: i32) -> String {
        match self {
            Unit::Fixed => {
                // Rounded on the magnitude. A shift floors, which rounds the
                // wrong way for negatives and turned -1.0 into "-1.001".
                let sign = if raw < 0 { "-" } else { "" };
                let m = ((raw as i64).abs() * 1000 + 32768) >> 16;
                let frac = format!("{:03}", m % 1000);
                let frac = frac.trim_end_matches('0');
                if frac.is_empty() {
                    format!("{sign}{}", m / 1000)
                } else {
                    format!("{sign}{}.{frac}", m / 1000)
                }
            }
            Unit::Flag => if raw != 0 { "on" } else { "off" }.to_string(),
            _ => raw.to_string(),
        }
    }
}

// ---------------------------------------------------------------------------
// Scalars: the universal rules
// ---------------------------------------------------------------------------

macro_rules! scalars {
    ($($variant:ident, $family:literal, $label:literal, $unit:ident, $lo:expr, $hi:expr;)*) => {
        /// A universal rule, applying to every fighter.
        #[derive(Clone, Copy, PartialEq, Eq, Debug)]
        pub enum Scalar { $($variant,)* }

        impl Scalar {
            pub const ALL: &'static [Scalar] = &[$(Scalar::$variant,)*];

            pub const fn family(self) -> &'static str {
                match self { $(Scalar::$variant => $family,)* }
            }

            pub const fn label(self) -> &'static str {
                match self { $(Scalar::$variant => $label,)* }
            }

            pub const fn unit(self) -> Unit {
                match self { $(Scalar::$variant => Unit::$unit,)* }
            }

            /// Sensible editing bounds, in raw units. Not correctness limits --
            /// the feel tests are what stop a value being wrong. These only keep
            /// a slider usable.
            pub const fn range(self) -> (i32, i32) {
                match self { $(Scalar::$variant => ($lo, $hi),)* }
            }
        }
    };
}

scalars! {
    MoveSpeed,        "Movement", "Walk speed",             Fixed,   fx(1,1),   fx(20,1);
    GuardMoveSpeed,   "Movement", "Walk speed, guarding",   Fixed,   0,         fx(10,1);
    CrouchMoveSpeed,  "Movement", "Walk speed, crouching",  Fixed,   0,         fx(10,1);
    TurnRate,         "Movement", "Turn rate",              Fixed,   fx(1,100), fx(1,1);
    GuardTurnRate,    "Movement", "Turn rate, guarding",    Fixed,   fx(1,100), fx(1,1);
    PokeMobility,     "Movement", "Poke mobility (%)",      Percent, 0,         100;
    HindranceDecay,   "Movement", "Hindrance decay per frame", Fixed, 0,        fx(1,1);
    JumpSpeed,        "Air",      "Takeoff speed",          Fixed,   fx(1,1),   fx(25,1);
    Gravity,          "Air",      "Gravity",                Fixed,   fx(-80,1), fx(-1,1);
    FallCap,          "Air",      "Terminal velocity",      Fixed,   fx(-60,1), fx(-1,1);
    JumpHoldGravity,  "Air",      "Gravity while held",     Fixed,   fx(1,10),  fx(1,1);
    JumpHoldFrames,   "Air",      "Sustain window",         Frames,  0,         60;
    AirAccel,         "Air",      "Air acceleration",       Fixed,   0,         fx(40,1);
    AirSpeedCap,      "Air",      "Air speed cap (x walk)", Fixed,   fx(1,1),   fx(4,1);
    GuardArcCos,      "Defence",  "Guard arc (cos)",        Fixed,   fx(-1,1),  fx(1,1);
    ParryWindow,      "Defence",  "Parry window",           Frames,  1,         20;
    ParryStagger,     "Defence",  "Parry stagger",          Frames,  1,         100;
    DodgeFrames,      "Defence",  "Dodge length",           Frames,  1,         60;
    DodgeIframes,     "Defence",  "Dodge invulnerability",  Frames,  0,         60;
    DodgeSpeed,       "Defence",  "Dodge speed",            Fixed,   fx(1,1),   fx(40,1);
    AirDodgeFrames,   "Defence",  "Airdodge length",        Frames,  1,         60;
    AirDodgeSpeed,    "Defence",  "Airdodge speed",         Fixed,   fx(1,1),   fx(40,1);
    KnockbackDecay,   "Defence",  "Knockback decay",        Fixed,   0,         fx(1,1);
    BodyRadius,       "Body",     "Body radius",            Fixed,   fx(1,10),  fx(2,1);
    BodyHeight,       "Body",     "Body height",            Fixed,   fx(1,2),   fx(4,1);
    CrouchHeightScale,"Body",     "Crouch height (x)",      Fixed,   fx(1,10),  fx(1,1);
    CastHeight,       "Body",     "Cast height",            Fixed,   fx(1,10),  fx(4,1);
    MaxHealth,        "Match",    "Max health",             Int,     100,       5000;
    RoundOverFrames,  "Match",    "Round-over pause",       Frames,  30,        600;
    AirStallDamp,     "Air",      "Aerial hang damping",    Fixed,   0,         fx(1,1);
    AirAttackBoost,   "Air",      "Aerial poke boost",      Fixed,   0,         fx(10,1);
    JumpReleaseCut,   "Air",      "Rise kept on release",   Fixed,   fx(1,10),  fx(1,1);
    EffectTickFrames, "Effects",  "Damage tick interval",   Frames,  1,         60;
    PillarBaseRadiusStart, "Effects", "Pillar base radius, new",  Fixed, fx(1,10), fx(6,1);
    PillarBaseRadius, "Effects",  "Pillar base radius, grown", Fixed, fx(1,10), fx(6,1);
    PillarBaseHeight, "Effects",  "Pillar base height",     Fixed,   fx(1,10),  fx(4,1);
    PillarRadiusStart,"Effects",  "Pillar column radius, new", Fixed, fx(1,10), fx(6,1);
    PillarColumnRadius,"Effects", "Pillar column radius, grown", Fixed, fx(1,10), fx(6,1);
    PillarHeightStart,"Effects",  "Pillar height, new",     Fixed,   fx(1,10),  fx(20,1);
    PillarHeight,     "Effects",  "Pillar height, grown",   Fixed,   fx(1,10),  fx(20,1);
    PillarLife,       "Effects",  "Pillar lifetime",        Frames,  10,        600;
    // The spike's radius, lifetime and drain went with the drain field: it
    // is a one-event spike now, sized by the move's own disc or the pool it
    // erupts from, standing for `SpikeErupt` frames under *Blood mage*.
    SpikeSlow,        "Effects",  "Black spike slow (x)",   Fixed,   0,         fx(1,1);
    StructureRadius,  "Stones",   "Stone radius",           Fixed,   fx(1,10),  fx(4,1);
    SlowFrames,       "Effects",  "Slow duration",          Frames,  1,         120;
    PillarDamage,     "Effects",  "Fire pillar tick",       Int,     0,         300;
    StructureRise,    "Stones",   "Rise",                   Frames,  1,         90;
    ShieldSpeed,      "Bulwark",  "Shield speed",               Fixed,   fx(1,1),   fx(40,1);
    ShieldRange,      "Bulwark",  "Shield range",               Fixed,   fx(1,1),   fx(30,1);
    ShieldRadius,     "Bulwark",  "Shield radius",              Fixed,   fx(1,10),  fx(3,1);
    ShieldDamage,     "Bulwark",  "Shield damage",              Int,     0,         600;
    ShieldHitstun,    "Bulwark",  "Shield hitstun",             Frames,  0,         90;
    ShieldBlockstun,  "Bulwark",  "Shield blockstun",           Frames,  0,         90;
    ShieldKnockback,  "Bulwark",  "Shield knockback",           Fixed,   0,         fx(30,1);
    LeapSpeed,        "Bulwark",  "Leap speed",                 Fixed,   fx(1,1),   fx(40,1);
    LeapRise,         "Bulwark",  "Leap rise",                  Fixed,   0,         fx(25,1);
    ShadowLeash,      "Reaver",   "Shadow leash",               Fixed,   fx(1,1),   fx(30,1);
    StructureAhead,   "Stones",   "Raise reach",                Fixed,   0,         fx(10,1);
    StructureHeight,  "Stones",   "Stone height",               Fixed,   fx(1,10),  fx(8,1);
    DodgeDecay,       "Defence",  "Dodge decay",                Fixed,   0,         fx(1,1);
    StunDecay,        "Defence",  "Hitstun decay",              Fixed,   0,         fx(1,1);
    SettleDecay,      "Match",    "Settle decay",               Fixed,   0,         fx(1,1);
    MeterMax,         "Dual mage","Meter range",                Int,     10,        400;
    MeterDeep,        "Dual mage","Meter deep threshold",       Int,     1,         400;
    MeterBurn,        "Dual mage","Burn at full depth",         Int,     0,         100;
    RiseCurveX1,      "Stones",   "Rise, hold",                 Fixed,   0,         fx(1,1);
    RiseCurveY1,      "Stones",   "Rise, hold lift",            Fixed,   0,         fx(1,1);
    RiseCurveX2,      "Stones",   "Rise, burst",                Fixed,   0,         fx(1,1);
    RiseCurveY2,      "Stones",   "Rise, burst lift",           Fixed,   0,         fx(1,1);
    StoneErupt,       "Stones",   "Eruption begins at",         Fixed,   0,         fx(1,1);
    StoneChurnSlow,   "Stones",   "Churn slow (x)",             Fixed,   0,         fx(1,1);
    StoneEruptDamage, "Stones",   "Eruption damage",            Int,     0,         300;
    StoneEruptStagger,"Stones",   "Eruption stagger",           Frames,  0,         90;
    StoneLift,        "Stones",   "Lift kept (x)",              Fixed,   0,         fx(2,1);
    StoneKnockHanded, "Stones",   "Knock handed on (x)",        Fixed,   0,         fx(1,1);
    StoneKnockDamp,   "Stones",   "Knock damping (x)",          Fixed,   0,         fx(1,1);
    StoneFriction,    "Stones",   "Ground friction (x)",        Fixed,   0,         fx(1,1);
    MonsterScale,     "Ridgeback","Size (x)",                   Fixed,   fx(1,2),   fx(3,1);
    MonsterHealth,    "Ridgeback","Health",                     Int,     500,       30000;
    LimbHealth,       "Ridgeback","Foot health",                Int,     100,       6000;
    MonsterMargin,    "Ridgeback","Keep-out from the wall",     Fixed,   0,         fx(12,1);
    MonsterWalk,      "Ridgeback","Walk speed",                 Fixed,   0,         fx(20,1);
    GallopSpeed,      "Ridgeback","Gallop speed",               Fixed,   fx(1,1),   fx(30,1);
    MonsterBack,      "Ridgeback","Backing-off speed",          Fixed,   0,         fx(12,1);
    MonsterAccel,     "Ridgeback","Walk acceleration",          Fixed,   fx(1,1),   fx(40,1);
    ProwlRange,       "Ridgeback","Preferred distance",         Fixed,   fx(1,1),   fx(20,1);
    ApproachGain,     "Ridgeback","Approach gain",              Fixed,   0,         fx(6,1);
    TurnRateMax,      "Ridgeback","Turn rate cap (turns/s)",    Fixed,   fx(1,100), fx(2,1);
    TurnGain,         "Ridgeback","Turn gain",                  Fixed,   fx(1,10),  fx(10,1);
    TurnAccel,        "Ridgeback","Turn acceleration",          Fixed,   fx(1,100), fx(10,1);
    TurnHurt,         "Ridgeback","Turn per broken leg (x)",    Fixed,   0,         fx(1,1);
    TurnSettle,       "Ridgeback","Turn bleed-off, committed",  Fixed,   0,         fx(1,1);
    GlanceFrames,     "Ridgeback · mind","Frames between glances", Frames, 1,       90;
    Lead,             "Ridgeback · mind","Lead on the target (x)", Fixed,  0,       fx(2,1);
    ProwlLead,        "Ridgeback · mind","Lead horizon, prowling", Frames, 0,       90;
    ThinkFrames,      "Ridgeback · mind","Pause between moves",    Frames, 0,       120;
    Decisiveness,     "Ridgeback · mind","Decisiveness (%)",       Percent, 0,      100;
    HurtAggression,   "Ridgeback · mind","Aggression when wounded", Int,   0,       400;
    VarietyPenalty,   "Ridgeback · mind","Repeat penalty",         Int,    0,       2000;
    VarietyFrames,    "Ridgeback · mind","Repeat penalty decay",   Frames, 0,       300;
    PoiseMax,         "Ridgeback","Poise",                      Int,     50,        5000;
    PoiseRegen,       "Ridgeback","Poise regained per frame",   Int,     0,         60;
    ToppleFrames,     "Ridgeback","Topple length",              Frames,  30,        600;
    StumbleFrames,    "Ridgeback","Stumble length",             Frames,  20,        300;
    FlinchFrames,     "Ridgeback","Flinch length",              Frames,  1,         90;
    FlinchThreshold,  "Ridgeback","Damage that flinches it",    Int,     1,         2000;
    VulnHead,         "Ridgeback · hide","Head (x damage)",     Fixed,   0,         fx(3,1);
    VulnNeck,         "Ridgeback · hide","Neck (x damage)",     Fixed,   0,         fx(3,1);
    VulnNape,         "Ridgeback · hide","Nape (x damage)",     Fixed,   0,         fx(4,1);
    VulnShoulder,     "Ridgeback · hide","Shoulders (x damage)",Fixed,   0,         fx(3,1);
    VulnBarrel,       "Ridgeback · hide","Barrel (x damage)",   Fixed,   0,         fx(3,1);
    VulnRidge,        "Ridgeback · hide","Ridge (x damage)",    Fixed,   0,         fx(4,1);
    VulnHaunch,       "Ridgeback · hide","Haunch (x damage)",   Fixed,   0,         fx(3,1);
    VulnTail,         "Ridgeback · hide","Tail (x damage)",     Fixed,   0,         fx(3,1);
    VulnTailMid,      "Ridgeback · hide","Tail, middle (x damage)", Fixed, 0,       fx(3,1);
    VulnTailTip,      "Ridgeback · hide","Tail tip (x damage)", Fixed,   0,         fx(3,1);
    VulnLeg,          "Ridgeback · hide","Upper leg (x damage)",Fixed,   0,         fx(3,1);
    VulnFoot,         "Ridgeback · hide","Foot (x damage)",     Fixed,   0,         fx(3,1);
    LegDrop,          "Ridgeback · legs","Corner drop per break", Fixed, 0,         fx(2,1);
    LegPitch,         "Ridgeback · legs","Pitch per break",     Fixed,   0,         fx(1,8);
    LegRoll,          "Ridgeback · legs","Roll per break",      Fixed,   0,         fx(1,8);
    LegFold,          "Ridgeback · legs","Broken leg, knee fold", Fixed, 0,         fx(1,4);
    LegBuckle,        "Ridgeback · legs","Broken leg, hip share (x)", Fixed, 0,    fx(2,1);
    LegSpeedHurt,     "Ridgeback · legs","Speed per break (x)", Fixed,   0,         fx(1,1);
    StrainDecay,      "Ridgeback · nerve","Strain bled per frame (%)", Percent, 1,  50;
    CcStrain,         "Ridgeback · nerve","Strain to feel control", Int,  50,        6000;
    InterruptStrain,  "Ridgeback · nerve","Strain to interrupt",    Int,  50,        9000;
    StrainDesperation,"Ridgeback · nerve","Thresholds fall by (%)", Percent, 0,      95;
    CcSlowBite,       "Ridgeback · nerve","Slow it actually feels (x)", Fixed, 0,    fx(1,1);
    CcRoot,           "Ridgeback · nerve","Root, per grab frame (x)",  Fixed, 0,     fx(2,1);
    CcStumble,        "Ridgeback · nerve","Stumble per launch (frames/mps)", Fixed, 0, fx(30,1);
    ShakeForce,       "Ridgeback · pose","Shake force (x)",     Fixed,   0,         fx(3,1);
    GaitStride,       "Ridgeback · pose","Stride length",       Fixed,   fx(1,2),   fx(12,1);
    BreathRate,       "Ridgeback · pose","Breath per frame",    Int,     1,         2000;
    HeadTrack,        "Ridgeback · pose","Head tracking (turns)", Fixed, 0,         fx(1,4);
    MountSnap,        "Riding",   "Landing reach",              Fixed,   fx(1,20),  fx(2,1);
    EdgeGrace,        "Riding",   "Overhang allowed (x body)",  Fixed,   0,         fx(2,1);
    Grip,             "Riding",   "Grip (m/s2)",                Fixed,   fx(20,1),  fx(2000,1);
    BraceGrip,        "Riding",   "Grip while bracing (x)",     Fixed,   fx(1,1),   fx(6,1);
    ThrowKick,        "Riding",   "Thrown, outward",            Fixed,   0,         fx(30,1);
    ThrowLift,        "Riding",   "Thrown, upward",             Fixed,   0,         fx(30,1);
    ThrowStun,        "Riding",   "Thrown, frames helpless",    Frames,  0,         90;
    ThrowDamage,      "Riding",   "Thrown, damage taken",       Int,     0,         400;
    MountSettle,      "Riding",   "Frames to plant your feet",  Frames,  1,         30;
    RiderSpeed,       "Riding",   "Walk speed aboard (x)",      Fixed,   fx(1,10),  fx(1,1);
    LeapCarry,        "Riding",   "Momentum a leap can carry",  Fixed,   0,         fx(40,1);
    MonsterSpawn,     "Ridgeback","Spawns this far out",        Fixed,   0,         fx(14,1);
    HunterSpawn,      "Ridgeback","Hunters start this far out", Fixed,   0,         fx(14,1);
    StepUp,           "Riding",   "Step you can walk up",       Fixed,   0,         fx(2,1);
    BoltKnockSpeed,    "Elementalist", "Bolt knock speed",                  Fixed,  0,        fx(60,1);
    BoltKnockRange,    "Elementalist", "Bolt knock travel",                 Fixed,  fx(1,10), fx(20,1);
    BoltKnockDecelStart,"Elementalist","Bolt knock decel starts (x path)",  Fixed,  0,        fx(1,1);
    BoltKnockMinSpeed, "Elementalist", "Bolt knock min speed to hurt",      Fixed,  0,        fx(30,1);
    BoltKnockDamagePerSpeed, "Elementalist", "Bolt knock damage per m/s",   Int,    0,        50;
    BoltKnockStagger,  "Elementalist", "Bolt knock stagger",                Frames, 0,        90;
    BoltKnockPush,     "Elementalist", "Bolt knock push (x)",               Fixed,  0,        fx(3,1);
    FireBoltSpeed,     "Elementalist", "Fire bolt speed",                   Fixed,  fx(5,1),  fx(80,1);
    FireBoltRange,     "Elementalist", "Fire bolt range",                   Fixed,  fx(1,1),  fx(40,1);
    FireBoltRadius,    "Elementalist", "Fire bolt radius",                  Fixed,  fx(1,20), fx(2,1);
    FireBoltDamage,    "Elementalist", "Fire bolt damage",                  Int,    0,        600;
    FireBoltStagger,   "Elementalist", "Fire bolt stagger",                 Frames, 0,        90;
    FireBoltBlockstun, "Elementalist", "Fire bolt blockstun",               Frames, 0,        90;
    FireBoltKnockback, "Elementalist", "Fire bolt knockback",               Fixed,  0,        fx(30,1);
    TornadoSpeed,      "Elementalist", "Fire tornado speed",                Fixed,  fx(1,1),  fx(40,1);
    TornadoPull,       "Elementalist", "Fire tornado pull (m/s2)",          Fixed,  0,        fx(100,1);
    TornadoTravelLife, "Elementalist", "Fire tornado travel time",          Frames, 1,        600;
    DebrisSpeed,       "Elementalist", "Cataclysm debris speed",            Fixed,  fx(1,1),  fx(40,1);
    DebrisRange,       "Elementalist", "Cataclysm debris range",            Fixed,  fx(1,1),  fx(20,1);
    DebrisRadius,      "Elementalist", "Cataclysm debris radius",           Fixed,  fx(1,10), fx(2,1);
    DebrisSpread,      "Elementalist", "Cataclysm debris fan (turns)",      Fixed,  0,        fx(1,4);
    DebrisDamage,      "Elementalist", "Cataclysm debris damage, per piece",Int,    0,        200;
    DebrisStagger,     "Elementalist", "Cataclysm debris stagger",          Frames, 0,        60;
    DebrisBlockstun,   "Elementalist", "Cataclysm debris blockstun",        Frames, 0,        30;
    DebrisKnockback,   "Elementalist", "Cataclysm debris knockback",        Fixed,  0,        fx(30,1);
    SpikeHeight,       "Blood mage", "Black spike height",                  Fixed,  fx(1,2),  fx(6,1);
    BloodletterFlight, "Blood mage", "Bloodletter, out and back",           Frames, 10,       180;
    BloodletterRadius, "Blood mage", "Bloodletter radius",                  Fixed,  fx(1,10), fx(2,1);
    GraspFlight,       "Blood mage", "Grasp, arms out and in",              Frames, 6,        120;
    GraspSpread,       "Blood mage", "Grasp, how wide the cone opens",      Fixed,  fx(1,10), fx(6,1);
    GraspArmRadius,    "Blood mage", "Grasp, arm radius",                   Fixed,  fx(1,10), fx(2,1);
    GraspBind,         "Blood mage", "Grasp, held still before the haul",   Frames, 0,        60;
    DisabledDamageMul, "Blood mage", "Damage to the disabled (x)",          Fixed,  fx(1,1),  fx(3,1);
    GraspMark,         "Blood mage", "Grasp, aim marker radius",            Fixed,  fx(1,20), fx(1,1);
    ReelSpeed,         "Defence",  "Grab haul speed",                       Fixed,  fx(1,1),  fx(80,1);
    RushSpeed,        "Champion", "Rush speed",                 Fixed,   fx(1,1),   fx(40,1);
    RushFrames,       "Champion", "Rush length",                Frames,  1,         60;
    RushRecharge,     "Champion", "Rush recharge",              Frames,  0,         240;
    VaultPitch,       "Champion", "Vault plants below (turns)", Fixed,   0,         fx(1,4);
    VaultCarry,       "Champion", "Vault keeps of the dash (x)",Fixed,   0,         fx(1,1);
    UppercutLeap,     "Champion", "Uppercut leap",              Fixed,   0,         fx(25,1);
    AirHitKnockback,  "Champion", "Knockback on an airborne target (x)", Fixed, fx(1,1), fx(4,1);
    SlamDamage,       "Champion", "Slam damage per m/s",        Int,     0,         40;
    SlamStagger,      "Champion", "Slam stagger",               Frames,  0,         90;
    SpearFanBoost,    "Champion", "Air spear boost on hit",     Fixed,   0,         fx(20,1);
    SweepHeight,      "Champion", "Sweep thrown from (x chest)",Fixed,   fx(1,10),  fx(3,2);
    SweepDip,         "Champion", "Sweep travels below level",  Fixed,   0,         fx(1,8);
    ThrustExtend,     "Champion", "Thrust out on the first active frame (x)", Fixed, 0, fx(1,1);
    ChainGrace,       "Champion", "Chain survives for",         Frames,  0,         120;
    ChainCancelSwap,  "Champion", "Chain, recovery owed on a swap (%)",     Percent, 0, 100;
    ChainCancelRepeat,"Champion", "Chain, recovery owed on a repeat (%)",   Percent, 0, 100;
    TakeoffWindow,    "Champion", "Takeoff window around a jump", Frames, 0,        30;
    PoleDriveBoost,   "Champion", "Pole drive, forward boost",  Fixed,   0,         fx(30,1);
    SwingLevelTo,      "Aim",       "Swing stays level to (deg down)",       Int,    0,        89;
    HandOffset,        "Body",      "Hand out from the centre line",         Fixed,  0,        fx(1,1);
    WingInner,         "Dual mage", "Wing, inner edge (x reach)",            Fixed,  0,        fx(1,1);
    WingTip,           "Dual mage", "Wing, the tip as a share of the span",  Fixed,  0,        fx(1,1);
    WingTipper,        "Dual mage", "Wing, tip damage (x)",                  Fixed,  fx(1,1),  fx(3,1);
    MeterAutoPush,     "Dual mage", "Meter, an auto moves",                  Int,    0,        100;
    MeterCastPush,     "Dual mage", "Meter, a cast moves",                   Int,    0,        100;
    AscensionFrames,   "Dual mage", "Ascension, how long",                   Frames, 1,        600;
    AscensionDrain,    "Dual mage", "Ascension, health a frame",             Int,    0,        100;
    AscensionStun,     "Dual mage", "Ascension, stun on the way out",        Frames, 0,        120;
    ShadowTrail,      "Reaver",   "Shadow trails her by",       Fixed,   0,         fx(4,1);
    ShadowFollow,     "Reaver",   "Shadow catch-up per frame",  Fixed,   fx(1,100), fx(1,1);
    ShadowLag,        "Reaver",   "Shadow copies her this late",Frames,  0,         30;
    ShadowEcho,       "Reaver",   "Shadow damage (%)",          Percent, 0,         100;
    ShadowSendFrames, "Reaver",   "Shadow flight, out",         Frames,  1,         60;
    ShadowHomeSpeed,  "Reaver",   "Shadow speed, coming home",  Fixed,   fx(1,1),   fx(60,1);
    ShadowRecallSlow, "Reaver",   "Recall slow (x)",            Fixed,   0,         fx(1,1);
    ShadowLockCone,   "Reaver",   "Crosshair lock on the shadow", Fixed, fx(1,10),  fx(6,1);
    ShadowDashSpeed,  "Reaver",   "Dash to the shadow, speed",  Fixed,   fx(1,1),   fx(40,1);
    LotusRadius,      "Reaver",   "Lotus, how far the blades go", Fixed, fx(1,1),   fx(12,1);
    // Was "how high they arc", when the blades left the shadow's feet and rose
    // over the eruption. The flower is flat now and this slot carries the plane
    // it lies in instead -- **renamed rather than removed**, because
    // `tuned::SCALARS` is read by this enum's own discriminant and dropping one
    // from the middle would hand every knob below it its neighbour's value.
    LotusHeight,      "Reaver",   "Lotus, height off the shadow's feet", Fixed, 0,   fx(4,1);
    LotusCurl,        "Reaver",   "Lotus, curve of the path (turns)", Fixed, 0,     fx(1,4);
    LotusBladeRadius, "Reaver",   "Lotus, blade radius",        Fixed,   fx(1,10),  fx(2,1);
    LotusErupt,       "Reaver",   "Lotus, out",                 Frames,  1,         60;
    LotusHold,        "Reaver",   "Lotus, held open",           Frames,  0,         120;
    LotusReturn,      "Reaver",   "Lotus, back to the shadow",  Frames,  1,         120;
    LotusReturnDamage,"Reaver",   "Lotus, damage coming home (%)", Percent, 0,      200;
    LotusSlow,        "Reaver",   "Lotus slow (x)",             Fixed,   0,         fx(1,1);
    // Appended, and they have to be: `tuned::SCALARS` is read by the enum's own
    // discriminant, so inserting one beside its family would silently give
    // every knob after it somebody else's baked value. The palette groups by
    // family rather than by position, so these still show up next to the three
    // wing knobs above.
    WingOffside,       "Dual mage", "Wing, ring centre off the punching side (x reach)", Fixed, fx(-1,1), fx(1,1);
    WingAhead,         "Dual mage", "Wing, ring centre ahead of her (x reach)", Fixed, fx(-1,1), fx(1,1);
    WingFinish,        "Dual mage", "Wing, finishes off centre (turns)",     Fixed,  fx(-1,4), fx(1,4);
    WingTipRadius,     "Dual mage", "Wing, tip radius",                      Fixed,  fx(1,20), fx(2,1);
    ShadowBuffer,      "Reaver",    "Send shadow, press stays live",         Frames, 1,        30;
    LotusUncurl,       "Reaver",    "Lotus, turn coming home (turns)",       Fixed,  0,        fx(1,2);
    RepeatLockout,     "Offence",   "Repeat lockout",                        Frames, 0,        90;
    LotusBladeThick,   "Reaver",    "Lotus, blade half-thickness",           Fixed,  fx(1,100), fx(1,2);
    ShadowCarry,       "Reaver",    "Dash carry, the jump window",           Frames, 0,        40;
    CommittedMobility, "Movement",  "Committed mobility (%)",                Percent, 0,       100;
    // The Elementalist's air row. Appended, like everything above them, because
    // `tuned::SCALARS` is read by this enum's own discriminant -- slotting one
    // in beside the other Elementalist knobs would hand every knob below it its
    // neighbour's baked value. The palette groups by family, so they still show
    // up next to the rest of hers.
    AirBoltSpeed,      "Elementalist", "Air bolt speed",                        Fixed,  fx(5,1),  fx(80,1);
    GaleSpeed,         "Elementalist", "Gale speed",                            Fixed,  fx(1,1),  fx(40,1);
    GaleStart,         "Elementalist", "Gale size leaving her hand (x)",        Fixed,  fx(1,20), fx(1,1);
    LandfallDive,      "Elementalist", "Landfall dive speed",                   Fixed,  fx(1,1),  fx(60,1);
    LandfallAhead,     "Elementalist", "Landfall stone, how far ahead",         Fixed,  fx(1,2),  fx(8,1);
    LandfallRise,      "Elementalist", "Landfall stone rise",                   Frames, 1,        90;
    LandfallTilt,      "Elementalist", "Landfall eruption, above the floor (turns)", Fixed, 0,    fx(1,4);
    LandfallErupt,     "Elementalist", "Landfall eruption push",                Fixed,  0,        fx(40,1);
    // Appended rather than slotted in beside `GaleStart`, which is where it
    // belongs by subject and where it may not go: `tuned::SCALARS` is read by
    // this enum's own discriminant, so inserting one in the middle hands every
    // knob below it its neighbour's baked value. The palette groups by family,
    // so it still shows up next to the rest of hers.
    GaleGrow,          "Elementalist", "Gale full size after (m)",              Fixed,  fx(1,1),  fx(40,1);
    // Appended for the same reason every knob above it is: `tuned::SCALARS` is
    // read by this enum's own discriminant, so a knob slotted in beside its
    // family would hand every knob below it its neighbour's baked value. The
    // palette groups by family, so these still show up where they belong.
    //
    // **The step's run-up** is shared across the whole roster, because the
    // *shape* of a step is a rule and only its size is a per-move decision:
    // every stepping move drives for this many frames plus its own active
    // window, and arrives exactly as the weapon does. See `moves::Move::step`.
    StepLead,         "Movement", "Step, frames of run-up before contact", Frames, 0, 20;
    // **How far a diagonal cut is rolled off the vertical**, in turns. An
    // eighth of a turn is corner to corner; nothing is upright and a quarter is
    // flat, so the two ends of this slider are the other two planes. One
    // magnitude for both of the sword's mirrored cuts -- the sign is which
    // shoulder it starts over. See `moves::Plane::Diagonal`.
    CutRoll,          "Champion", "Diagonal cut, roll off the vertical (turns)", Fixed, 0, fx(1,4);
    // **Going up with them.** Jump during the hammer's finisher and the
    // knock-up is worth more and you leave the floor on the frame it lands, so
    // the pair of you arrive in the air together. Two numbers because they are
    // two decisions: how much harder they go, and how fast you follow.
    HammerLeapLaunch, "Champion", "Hammer finisher, knock-up going with them (x)", Fixed, fx(1,1), fx(3,1);
    HammerLeap,       "Champion", "Hammer finisher, your own leap",         Fixed, 0, fx(25,1);
    // The Dual mage's depth curve, and everything downstream of it. Appended
    // for the reason every knob above is: `tuned::SCALARS` is read by this
    // enum's own discriminant. The palette groups by family, so they sit with
    // the wing knobs.
    //
    // **Two numbers describe the whole class.** Power is a straight line from
    // the centre of the bar to either end, and these are its two ends: what a
    // cast is worth standing at zero, and what the same cast is worth standing
    // at the edge. Everything she throws is multiplied by a point on that line
    // -- see `state::depth`.
    DepthFloor,       "Dual mage", "Depth, power at the centre (x)",        Fixed, fx(1,10), fx(1,1);
    DepthCeiling,     "Dual mage", "Depth, power at the edge (x)",          Fixed, fx(1,1),  fx(4,1);
    // How much of that same curve the *size* of a volume takes. At nought a
    // deep cast hits harder and is no bigger; at a hundred it grows exactly as
    // fast as it hurts. The two autos are exempt whatever this says -- their
    // reach is pinned to the punch that throws them, see `moves::dual`.
    DepthSize,        "Dual mage", "Depth, how much of it the size takes (%)", Percent, 0, 100;
    // The third meter tier, and the one the design has been asking for since
    // the bar was built: an auto moves you a little, a cast moves you more, and
    // the finisher very nearly throws you over the edge.
    MeterFinisherPush,"Dual mage", "Meter, the finisher moves",             Int,   0,        100;
    // What the wing's tip does to the *shove* rather than to the damage. The
    // tip is the far edge of the blade, so it is where the light auto's real
    // knockback lives -- and, on the dark arm, where the pull is worth timing.
    WingTipShove,     "Dual mage", "Wing, tip knockback (x)",               Fixed, fx(1,1),  fx(4,1);
    // Sweep's dark form. The light form spends the move's own knockback and
    // launch; the dark one spends these instead, which is what makes one
    // button two answers to the same problem.
    SweepSlow,        "Dual mage", "Sweep, dark: speed while slowed (x)",   Fixed, 0,        fx(1,1);
    SweepHeal,        "Dual mage", "Sweep, dark: health per target caught", Int,   0,        200;
    // The light Lance's burst: what detonates where the line ran out. Aimed
    // *past* somebody rather than at them, which is the whole reading of the
    // move.
    LanceBurstRadius, "Dual mage", "Light lance, burst radius",             Fixed, fx(1,2),  fx(8,1);
    LanceBurstDamage, "Dual mage", "Light lance, burst damage",             Int,   0,        400;
    LanceBurstLife,   "Dual mage", "Light lance, burst lingers",            Frames, 1,       60;
    // The dark Lance's tether: how long it can hold, what it takes per tick,
    // and how far she may stray before the line parts. The leash is the whole
    // cost of the ability -- it drains for as long as she stays near what she
    // caught, which on a fragile melee mage is not a free decision.
    TetherLife,       "Dual mage", "Dark lance, tether holds for",          Frames, 6,       300;
    TetherDrain,      "Dual mage", "Dark lance, drain per tick",            Int,    0,       120;
    TetherLeash,      "Dual mage", "Dark lance, leash before it parts",     Fixed,  fx(2,1), fx(24,1);
    // Judgement's field: the wide, low-damage ground it leaves after the
    // strike. Its radius and its life are both on the depth curve, so at the
    // centre it is a puddle that is gone before anyone walks through it.
    JudgementFieldRadius, "Dual mage", "Judgement, field radius",           Fixed,  fx(1,1), fx(12,1);
    JudgementFieldDamage, "Dual mage", "Judgement, field damage per tick",  Int,    0,       120;
    JudgementFieldLife,   "Dual mage", "Judgement, field lasts",            Frames, 1,       300;
    JudgementFieldSpeed,  "Dual mage", "Judgement, her speed inside it (x)", Fixed, fx(1,1), fx(3,1);
    // The Blood mage's rebuild, appended: grey health, the scythe that grows
    // with it, and the essence pools the other fighter bleeds onto the floor.
    // See `docs/design/blood-mage.md`.
    GreyFade,         "Blood mage", "Grey fades (health per second)",         Int,   0,        600;
    GreyReach,        "Blood mage", "Scythe reach at full grey (x)",          Fixed, fx(1,1),  fx(4,1);
    GreyDamage,       "Blood mage", "Scythe damage at full grey (x)",         Fixed, fx(1,1),  fx(4,1);
    SweepTip,         "Blood mage", "Sweep, tip as a share of the reach",     Fixed, 0,        fx(1,1);
    SweepTipDamage,   "Blood mage", "Sweep, tip damage (x)",                  Fixed, fx(1,1),  fx(4,1);
    PoolDrain,        "Blood mage", "Pool drains (volume per second)",        Int,   0,        600;
    PoolCap,          "Blood mage", "Pools at once",                          Int,   1,        8;
    PoolLock,         "Blood mage", "Crosshair lock on a pool",               Fixed, 0,        fx(6,1);
    SpikeErupt,       "Blood mage", "Black spike, eruption lasts",            Frames, 1,       120;
    // A pool is a shadowy figure the size of the body it came out of, and it
    // shrinks as it drains: full-sized at this much volume, never smaller
    // than this share of a body. Replaced a radius-per-root-of-volume and a
    // fixed slab height, which spread a Reap's pool across two and a half
    // metres of floor.
    PoolFull,         "Blood mage", "Pool, full-sized at volume",             Int,   1,        1000;
    PoolLeast,        "Blood mage", "Pool, smallest share of a body (x)",     Fixed, fx(1,20), fx(1,1);
    // What a spike cast on a pool becomes: an eruption with its own size and
    // its own weight, so that the placement the whole kit exists to arrange
    // is a different event from a spike on bare floor.
    EruptRadius,      "Blood mage", "Black spike, eruption radius per root of volume", Fixed, fx(1,100), fx(2,1);
    EruptHeight,      "Blood mage", "Black spike, eruption height",           Fixed, fx(1,2),  fx(10,1);
    EruptDamage,      "Blood mage", "Black spike, eruption damage (x)",       Fixed, fx(1,1),  fx(4,1);
}

// ---------------------------------------------------------------------------
// The view: numbers that decide what you see, not what happens
// ---------------------------------------------------------------------------

macro_rules! view_knobs {
    ($($variant:ident, $label:literal, $unit:ident, $lo:expr, $hi:expr;)*) => {
        /// A camera number.
        ///
        /// Separate from [`Scalar`] only as a grouping now. These **are** in
        /// [`hash`], like everything else that decides what happens.
        ///
        /// They were kept out of it for a while, on the reasoning that a camera
        /// decides what you *see* rather than what happens, so two people ought
        /// to be able to play each other with different framing. That was true
        /// exactly as long as aiming did not go through the camera.
        ///
        /// It does now: the crosshair is the aim, so the ray that decides where
        /// an ability lands starts at the eye, and where the eye sits is these
        /// numbers (`crate::camera`). A camera number is a gameplay number the
        /// moment the crosshair means something.
        #[derive(Clone, Copy, PartialEq, Eq, Debug)]
        pub enum ViewKnob { $($variant,)* }

        impl ViewKnob {
            pub const ALL: &'static [ViewKnob] = &[$(ViewKnob::$variant,)*];

            pub const fn label(self) -> &'static str {
                match self { $(ViewKnob::$variant => $label,)* }
            }

            pub const fn unit(self) -> Unit {
                match self { $(ViewKnob::$variant => Unit::$unit,)* }
            }

            pub const fn range(self) -> (i32, i32) {
                match self { $(ViewKnob::$variant => ($lo, $hi),)* }
            }
        }
    };
}

// The camera is described as a set of **zones** in vertical aim angle, with the
// waypoints between them stated as where the fighter should appear on screen.
// The zone boundaries and the numbers inside each zone are here; the *shape* --
// which waypoints exist and what each one means -- is in `view::camera` and is
// not a knob, because it is the design rather than a value.
//
// Angles are degrees, negative being below the horizon. Screen fractions are
// percentages measured from the bottom of the screen, so the crosshair sits at
// 50 by definition.
//
// The camera is always on the surface of a sphere, looking inward past a tilt,
// and the mouse walks it around that sphere at a steady rate. What the zones
// change is the sphere: where it is centred, how big it is, and how far the
// view is tilted off the line to its centre. The **radius** is the distance the
// camera keeps and therefore how big the fighter is drawn; the **screen
// percentages** are the tilts, written as where the point at the sphere's
// centre should sit on screen, which is the same thing.
//
// The **smoothing** percentages are what stops a zone boundary reading as the
// camera changing its mind. Each is a share of its own zone's angular span: how
// much of that zone's ramp, at each end, is spent easing in and out. The eye
// leaves and arrives at a standstill, so a neighbouring zone that is already
// holding still has nothing to hand over against. Outside that window the ramp is
// untouched, so the waypoints stay exactly as written; at the maximum the ramp is
// eased the whole way through and its middle still lands on the waypoint, because
// the two ends give back what each other took.
//
// Two of the five have nothing to ease yet. The neutral zone and first person
// both hold their sphere still and let the mouse do all the moving, so there is
// no ramp in them to shape -- their numbers are there for when there is.
view_knobs! {
    Sphere,         "Sphere radius (m)",        Fixed,   fx(1,2), fx(30,1);
    HeadSphere,     "Sphere, at the head (m)",  Fixed,   0,       fx(3,1);
    LookDownLimit,  "Look down limit",          Int,     10,  89;
    LookUpLimit,    "Look up limit",            Int,     10,  89;
    FloorZoneFrom,  "Floor zone, from",         Int,     10,  89;
    NeutralZoneTo,  "Neutral zone, to",         Int,     1,   45;
    HeadLockAt,     "Head lock, at",            Int,     1,   45;
    FeetNeutral,    "Feet, neutral (%)",        Int,     0,   50;
    FeetFloor,      "Feet, floor (%)",          Int,     0,   50;
    HeadGapLevel,   "Head to crosshair (%)",    Int,     0,   30;
    FadeNear,       "Body gone within (m)",     Fixed,   0,       fx(6,1);
    FramingFov,     "Framing field of view",    Int,     30,  120;
    SmoothFloor,    "Smooth, floor zone (%)",   Int,     0,   100;
    SmoothNeutral,  "Smooth, neutral zone (%)", Int,     0,   100;
    SmoothTurn,     "Smooth, the turn (%)",     Int,     0,   100;
    SmoothHandover, "Smooth, handover (%)",     Int,     0,   100;
    SmoothEyes,     "Smooth, first person (%)", Int,     0,   100;
    CrosshairDim,   "Body dims to (%)",         Int,     0,   100;
    // Appended rather than placed beside the other angles, for the reason the
    // scalars above give: `tuned::VIEW` is read by this enum's own
    // discriminant, so slotting one in beside its family would hand every knob
    // below it its neighbour's baked value.
    //
    // The range runs past the floor boundary on purpose. Dragging it there and
    // seeing the view pitch toward your own feet is how you find out why the
    // opening angle belongs inside the neutral zone; what stops it being
    // *committed* there is `view/tests/camera_knobs.rs`, not the slider.
    StartPitch,     "Opening pitch, below level", Int,   1,   60;
    // **The airborne framing**, appended for the reason everything above it is:
    // `tuned::VIEW` is read by this enum's own discriminant, so slotting one in
    // beside its relatives would hand every knob below it its neighbour's baked
    // value.
    //
    // Off the ground the camera swings over to framing the fighter's head just
    // under the crosshair, and how far over it has swung is a function of how
    // high above the floor they are -- an S-curve in height, rate-limited in
    // time so that a long fall does not whip the view around. See
    // `camera::aloft_target` and `camera::aloft_step`.
    AloftFull,      "Airborne framing, full at (m)", Fixed, fx(1,2), fx(20,1);
    AloftRate,      "Airborne framing, most it takes hold a frame", Fixed, fx(1,1000), fx(1,1);
    AloftEase,      "Airborne framing, most it lets go a frame",   Fixed, fx(1,1000), fx(1,1);
    AloftCurveX1,   "Airborne framing, curve: leaves",      Fixed, 0, fx(1,1);
    AloftCurveY1,   "Airborne framing, curve: leaves lift", Fixed, 0, fx(1,1);
    AloftCurveX2,   "Airborne framing, curve: arrives",     Fixed, 0, fx(1,1);
    AloftCurveY2,   "Airborne framing, curve: arrives lift",Fixed, 0, fx(1,1);
}

/// Counted from the table rather than written down, the same way
/// [`SCALAR_COUNT`] is. A literal here is a second statement of how many camera
/// knobs there are, and the two disagree the first time somebody appends one --
/// which shows up as a type error on the baked array rather than as anything
/// that names the cause.
pub const VIEW_COUNT: usize = ViewKnob::ALL.len();

// ---------------------------------------------------------------------------
// Per-class air, and per-move frame data
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum AirField {
    Jump,
    Gravity,
    FallCap,
    AirSpeed,
}

impl AirField {
    pub const ALL: &'static [AirField] = &[
        AirField::Jump,
        AirField::Gravity,
        AirField::FallCap,
        AirField::AirSpeed,
    ];

    pub const fn label(self) -> &'static str {
        match self {
            AirField::Jump => "Jump (x)",
            AirField::Gravity => "Gravity (x)",
            AirField::FallCap => "Fall cap (x)",
            AirField::AirSpeed => "Steering",
        }
    }

    pub const fn range(self) -> (i32, i32) {
        match self {
            AirField::AirSpeed => (0, fx(5, 1)),
            _ => (fx(3, 10), fx(5, 2)),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MoveField {
    Startup,
    Active,
    Recovery,
    Damage,
    Reach,
    Radius,
    Hitstun,
    Blockstun,
    Knockback,
    Mobility,
    AirStall,
    Unblockable,
    HitsCrouching,
    NeedsMechanic,
    // Appended, so field indices 0..13 keep the meaning the baked file was
    // written with. Only the stride changes, which the migration handled.
    Launch,
    SelfLift,
    Grabs,
    Effect,
    // Appended again, for the Blood mage's economy: health out on the press,
    // and health back on the hit -- see `moves::Move::cost` and `leech`. The
    // second half stopped being hers when the heal became a place on the
    // floor (see `Drink`, appended at the end); it stays a column because the
    // Dual mage's dark arm reads it.
    Cost,
    Leech,
    // And again, for which line of effect a move uses: 0 swings out along the
    // body, 1 lands on the ground at the crosshair, 2 flies to what the
    // crosshair is on, 3 erupts at the class mechanic. `aim::Kind`'s own
    // numbering. See `moves::Move::aim` and `crate::aim`.
    Aim,
    // And again for the Champion's rebuild: how far a swing travels, and how
    // often a move that keeps hitting is allowed to hit again.
    Arc,
    Rehit,
    // Appended for the first channelled move. Both are questions the rest of
    // the row already answers for an ordinary one -- how long it runs, how far
    // it goes -- asked about the wind-up instead.
    Channel,
    ChannelFrom,
    // And again for the repeat lockout: this move's own share of the global
    // one, as a percentage. See `moves::Move::repeat_lock`.
    RepeatMul,
    // And the other half of that rule, for the abilities that are used more
    // than once per cast: the shortest gap between one activation and the next.
    // See `moves::Move::reactivate`.
    Reactivate,
    // Appended, like everything above it: how far the move itself carries the
    // body forward. See `moves::Move::step`.
    Step,
    // Appended for the Blood mage's rebuild: what share of an essence pool a
    // move drinks when it lands over one. See `moves::Move::drink`.
    Drink,
}

impl MoveField {
    pub const ALL: &'static [MoveField] = &[
        MoveField::Startup,
        MoveField::Active,
        MoveField::Recovery,
        MoveField::Damage,
        MoveField::Reach,
        MoveField::Radius,
        MoveField::Hitstun,
        MoveField::Blockstun,
        MoveField::Knockback,
        MoveField::Mobility,
        MoveField::AirStall,
        MoveField::Unblockable,
        MoveField::HitsCrouching,
        MoveField::NeedsMechanic,
        MoveField::Launch,
        MoveField::SelfLift,
        MoveField::Grabs,
        MoveField::Effect,
        MoveField::Cost,
        MoveField::Leech,
        MoveField::Aim,
        MoveField::Arc,
        MoveField::Rehit,
        MoveField::Channel,
        MoveField::ChannelFrom,
        MoveField::RepeatMul,
        MoveField::Reactivate,
        MoveField::Step,
        MoveField::Drink,
    ];

    pub const fn label(self) -> &'static str {
        match self {
            MoveField::Startup => "Startup",
            MoveField::Active => "Active",
            MoveField::Recovery => "Recovery",
            MoveField::Damage => "Damage",
            MoveField::Reach => "Reach",
            MoveField::Radius => "Radius",
            MoveField::Hitstun => "Hitstun",
            MoveField::Blockstun => "Blockstun",
            MoveField::Knockback => "Knockback",
            MoveField::Mobility => "Mobility (%)",
            MoveField::AirStall => "Aerial hang",
            MoveField::Unblockable => "Unblockable",
            MoveField::HitsCrouching => "Hits crouching",
            MoveField::NeedsMechanic => "Needs mechanic",
            MoveField::Launch => "Launch",
            MoveField::SelfLift => "Self lift",
            MoveField::Grabs => "Grab hold",
            MoveField::Effect => "Leaves behind",
            MoveField::Cost => "Health cost",
            MoveField::Leech => "Leech (%)",
            MoveField::Aim => "Line of effect (0-3)",
            MoveField::Arc => "Swing arc (turns)",
            MoveField::Rehit => "Hits again every",
            MoveField::Channel => "Channel, longest hold",
            MoveField::ChannelFrom => "Channel, reach at no hold",
            MoveField::RepeatMul => "Repeat lockout (%)",
            MoveField::Reactivate => "Reactivate no sooner than",
            MoveField::Step => "Steps forward (m)",
            MoveField::Drink => "Drinks of a pool (%)",
        }
    }

    pub const fn unit(self) -> Unit {
        match self {
            MoveField::Startup
            | MoveField::Active
            | MoveField::Recovery
            | MoveField::Hitstun
            | MoveField::Blockstun
            | MoveField::AirStall => Unit::Frames,
            MoveField::Damage => Unit::Int,
            MoveField::Mobility => Unit::Percent,
            MoveField::Unblockable | MoveField::HitsCrouching | MoveField::NeedsMechanic => {
                Unit::Flag
            }
            MoveField::Grabs | MoveField::Rehit | MoveField::Channel | MoveField::Reactivate => {
                Unit::Frames
            }
            MoveField::Effect | MoveField::Aim => Unit::Int,
            MoveField::Cost | MoveField::Leech | MoveField::Drink | MoveField::RepeatMul => {
                Unit::Percent
            }
            _ => Unit::Fixed,
        }
    }

    pub const fn range(self) -> (i32, i32) {
        match self {
            // Four lines of effect, and the numbering is `aim::Kind`'s. A
            // slider that ran to six hundred would let somebody pick a fifth.
            MoveField::Aim => (0, 3),
            // Signed, because a spike is a launch pointed the other way: the
            // Champion's aerial hammer drives an airborne target into the
            // floor with the same number that an uppercut lifts them with.
            MoveField::Launch => (fx(-30, 1), fx(30, 1)),
            // Signed for the same reason in the other axis: the sign is which
            // way the weapon travels. See `moves::Move::arc`.
            //
            // A whole turn either way, because a spin is a real shape: the
            // spear's finisher carries the pole round past the shoulder it
            // started on, and a slider that stopped at half a turn could not
            // express it. Half was the old bound and it was set by the widest
            // thing in the game at the time, not by anything geometric.
            MoveField::Arc => (fx(-1, 1), fx(1, 1)),
            // Signed, and in metres: a move may give ground as well as take it.
            // The bound is about what a body can cross inside a handful of
            // frames -- see `moves::Move::step`.
            MoveField::Step => (fx(-3, 1), fx(4, 1)),
            // A takeoff speed, in the same units the jump is: the pole vault
            // is meant to beat a jump, and a jump is already 17.7.
            MoveField::SelfLift => (0, fx(30, 1)),
            // A speed, in the units every other knockback in the Oven is
            // written in -- `ShieldKnockback`, `FireBoltKnockback` and
            // `DebrisKnockback` all run to thirty, and this one ran to twelve
            // only because it inherited the shared `Fixed` bound below. A move
            // that shoves harder than a thrown shield is a thing somebody
            // should be able to reach for; the Gale, whose whole point is
            // being the heaviest push in its class, is the first that does.
            //
            // **Signed, and the negative half is a pull.** A move with a
            // knockback below zero drags whoever it caught *toward* the
            // attacker, along the line between the two bodies rather than down
            // the attacker's facing -- see `state::resolve_hit`. The Dual
            // mage's dark auto is the first and the reason: a fragile melee
            // mage whose left hand pulls and whose right hand pushes has a
            // spacing decision on the two buttons she presses constantly.
            MoveField::Knockback => (fx(-30, 1), fx(30, 1)),
            // A reach may be as far as a shot fired corner to corner has to
            // travel, and no further: past that, more range is a number that
            // cannot change anything. That is the **diagonal** rather than the
            // width -- `arena::ARENA_HALF` is fourteen, so the floor is
            // twenty-eight across and a little under forty corner to corner.
            //
            // The shared `Fixed` bound below is twelve metres, which was every
            // move's answer right up until a class was given something to throw
            // with its feet off the floor: what being airborne buys the
            // Elementalist is reach, and both of her air shots want more of it
            // than a grounded move has ever asked for.
            MoveField::Reach => (0, fx(40, 1)),
            // Past 100, unlike every other percentage here: this one *scales*
            // the shared lockout rather than taking a share of something, and
            // a move worth locking for twice as long as the rest is the first
            // thing anybody will reach for. Zero is the other end and is a real
            // setting -- it exempts a move from the rule entirely.
            MoveField::RepeatMul => (0, 300),
            _ => match self.unit() {
                Unit::Frames => (0, 90),
                Unit::Int => (0, 600),
                Unit::Percent => (0, 100),
                Unit::Flag => (0, 1),
                Unit::Fixed => (0, fx(12, 1)),
            },
        }
    }
}

// ---------------------------------------------------------------------------
// The Ridgeback's move data
// ---------------------------------------------------------------------------

/// One tuned field of one of the creature's moves.
///
/// Its own family rather than a reuse of `MoveField`, because the two tables
/// mean different things: a fighter's move is authored against a shared body
/// and a shared control scheme, and a creature's is authored in the creature's
/// own coordinates and carries what the control algorithm reads. Sharing the
/// enum would have meant a dozen fields on each side that the other ignores.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MonsterField {
    Startup,
    Active,
    Recovery,
    Damage,
    /// Where the hit volume sits, in the creature's own frame.
    HitX,
    HitZ,
    HitRadius,
    HitLow,
    HitHigh,
    /// Which articulation carries the volume: the tail's swing, or the head's
    /// reach. See `monster::Shape::rides`.
    Follows,
    Hitstun,
    Blockstun,
    Knockback,
    Launch,
    Unblockable,
    /// Forward speed while active. Only the charge has one.
    Advance,
    IdealRange,
    RangeSpan,
    AimCos,
    AimSpan,
    Weight,
    RiderWeight,
    /// Frames before this move can come back. What makes baiting one worth it.
    Cooldown,
}

impl MonsterField {
    pub const ALL: &'static [MonsterField] = &[
        MonsterField::Startup,
        MonsterField::Active,
        MonsterField::Recovery,
        MonsterField::Damage,
        MonsterField::HitX,
        MonsterField::HitZ,
        MonsterField::HitRadius,
        MonsterField::HitLow,
        MonsterField::HitHigh,
        MonsterField::Follows,
        MonsterField::Hitstun,
        MonsterField::Blockstun,
        MonsterField::Knockback,
        MonsterField::Launch,
        MonsterField::Unblockable,
        MonsterField::Advance,
        MonsterField::IdealRange,
        MonsterField::RangeSpan,
        MonsterField::AimCos,
        MonsterField::AimSpan,
        MonsterField::Weight,
        MonsterField::RiderWeight,
        MonsterField::Cooldown,
    ];

    pub const fn label(self) -> &'static str {
        match self {
            MonsterField::Startup => "Startup",
            MonsterField::Active => "Active",
            MonsterField::Recovery => "Recovery",
            MonsterField::Damage => "Damage",
            MonsterField::HitX => "Hit, forward",
            MonsterField::HitZ => "Hit, sideways",
            MonsterField::HitRadius => "Hit radius",
            MonsterField::HitLow => "Hit, lowest",
            MonsterField::HitHigh => "Hit, highest",
            MonsterField::Follows => "Carried by",
            MonsterField::Hitstun => "Hitstun",
            MonsterField::Blockstun => "Blockstun",
            MonsterField::Knockback => "Knockback",
            MonsterField::Launch => "Launch",
            MonsterField::Unblockable => "Unblockable",
            MonsterField::Advance => "Advance while active",
            MonsterField::IdealRange => "Ideal range",
            MonsterField::RangeSpan => "Range tolerance",
            MonsterField::AimCos => "Wants the target at (cos)",
            MonsterField::AimSpan => "Bearing tolerance",
            MonsterField::Weight => "Appetite",
            MonsterField::RiderWeight => "Appetite per rider",
            MonsterField::Cooldown => "Lockout after use",
        }
    }

    pub const fn unit(self) -> Unit {
        match self {
            MonsterField::Startup
            | MonsterField::Active
            | MonsterField::Recovery
            | MonsterField::Hitstun
            | MonsterField::Blockstun
            | MonsterField::Cooldown => Unit::Frames,
            MonsterField::Damage
            | MonsterField::Follows
            | MonsterField::Weight
            | MonsterField::RiderWeight => Unit::Int,
            MonsterField::Unblockable => Unit::Flag,
            _ => Unit::Fixed,
        }
    }

    pub const fn range(self) -> (i32, i32) {
        match self {
            MonsterField::Startup | MonsterField::Active | MonsterField::Recovery => (0, 120),
            MonsterField::Cooldown => (0, 600),
            MonsterField::Hitstun | MonsterField::Blockstun => (0, 90),
            MonsterField::Damage => (0, 900),
            MonsterField::Follows => (0, 2),
            MonsterField::Weight | MonsterField::RiderWeight => (0, 4000),
            MonsterField::Unblockable => (0, 1),
            // Body-space coordinates run behind the creature as well as ahead
            // of it, and the sweep's preferred bearing is negative.
            MonsterField::HitX | MonsterField::HitZ | MonsterField::AimCos => {
                (fx(-10, 1), fx(10, 1))
            }
            MonsterField::AimSpan => (fx(1, 100), fx(8, 1)),
            _ => (0, fx(40, 1)),
        }
    }
}

pub const MONSTER_MOVES: usize = 6;
pub const MONSTER_FIELDS: usize = 23;
pub const MONSTER_COUNT: usize = MONSTER_MOVES * MONSTER_FIELDS;

pub const CLASSES: usize = 6;
/// Counted from the table rather than written down. A literal here is a second
/// statement of how many knobs there are, and the two disagree the first time
/// somebody appends one -- which shows up as a type error on the baked array
/// rather than as anything that names the cause.
pub const SCALAR_COUNT: usize = Scalar::ALL.len();
pub const AIR_COUNT: usize = CLASSES * 4;
/// Move storage is packed to each class's own slot count rather than to a
/// single width. The Champion has ten moves, the Blood mage four and everybody
/// else three, and a rectangular table would have meant seven empty rows per
/// class in the palette and in the baked file.
pub const MOVE_COUNT: usize = crate::moves::TOTAL_SLOTS * MOVE_FIELDS;
pub const MOVE_FIELDS: usize = 29;

// ---------------------------------------------------------------------------
// The live store
// ---------------------------------------------------------------------------

fn cells<const N: usize>(seed: &'static [i32; N]) -> [AtomicI32; N] {
    std::array::from_fn(|i| AtomicI32::new(seed[i]))
}

static SCALAR_CELLS: LazyLock<[AtomicI32; SCALAR_COUNT]> = LazyLock::new(|| cells(&tuned::SCALARS));
static AIR_CELLS: LazyLock<[AtomicI32; AIR_COUNT]> = LazyLock::new(|| cells(&tuned::AIR));
static MOVE_CELLS: LazyLock<[AtomicI32; MOVE_COUNT]> = LazyLock::new(|| cells(&tuned::MOVES));
static VIEW_CELLS: LazyLock<[AtomicI32; VIEW_COUNT]> = LazyLock::new(|| cells(&tuned::VIEW));

pub fn view(k: ViewKnob) -> i32 {
    VIEW_CELLS[k as usize].load(Ordering::Relaxed)
}

pub fn set_view(k: ViewKnob, raw: i32) {
    VIEW_CELLS[k as usize].store(raw, Ordering::Relaxed);
}

static MONSTER_CELLS: LazyLock<[AtomicI32; MONSTER_COUNT]> =
    LazyLock::new(|| cells(&tuned::MONSTER));

pub fn scalar(s: Scalar) -> i32 {
    SCALAR_CELLS[s as usize].load(Ordering::Relaxed)
}

pub fn set_scalar(s: Scalar, raw: i32) {
    SCALAR_CELLS[s as usize].store(raw, Ordering::Relaxed);
}

fn air_index(class: Class, field: AirField) -> usize {
    class as usize * 4 + field as usize
}

pub fn air(class: Class, field: AirField) -> i32 {
    AIR_CELLS[air_index(class, field)].load(Ordering::Relaxed)
}

pub fn set_air(class: Class, field: AirField, raw: i32) {
    AIR_CELLS[air_index(class, field)].store(raw, Ordering::Relaxed);
}

fn move_index(class: Class, slot: usize, field: MoveField) -> usize {
    (crate::moves::base_slot(class) + slot) * MOVE_FIELDS + field as usize
}

pub fn move_field(class: Class, slot: usize, field: MoveField) -> i32 {
    MOVE_CELLS[move_index(class, slot, field)].load(Ordering::Relaxed)
}

pub fn set_move_field(class: Class, slot: usize, field: MoveField, raw: i32) {
    MOVE_CELLS[move_index(class, slot, field)].store(raw, Ordering::Relaxed);
}

fn monster_index(slot: usize, field: MonsterField) -> usize {
    slot * MONSTER_FIELDS + field as usize
}

pub fn monster_field(slot: usize, field: MonsterField) -> i32 {
    MONSTER_CELLS[monster_index(slot, field)].load(Ordering::Relaxed)
}

pub fn set_monster_field(slot: usize, field: MonsterField, raw: i32) {
    MONSTER_CELLS[monster_index(slot, field)].store(raw, Ordering::Relaxed);
}

/// Put everything back to what is committed in `tuned.rs`.
pub fn reset_to_baked() {
    for (i, cell) in SCALAR_CELLS.iter().enumerate() {
        cell.store(tuned::SCALARS[i], Ordering::Relaxed);
    }
    for (i, cell) in AIR_CELLS.iter().enumerate() {
        cell.store(tuned::AIR[i], Ordering::Relaxed);
    }
    for (i, cell) in MOVE_CELLS.iter().enumerate() {
        cell.store(tuned::MOVES[i], Ordering::Relaxed);
    }
    for (i, cell) in VIEW_CELLS.iter().enumerate() {
        cell.store(tuned::VIEW[i], Ordering::Relaxed);
    }
    for (i, cell) in MONSTER_CELLS.iter().enumerate() {
        cell.store(tuned::MONSTER[i], Ordering::Relaxed);
    }
}

/// True when the live values differ from what is committed -- what the bake
/// button needs to know, and what stops an empty commit.
pub fn is_dirty() -> bool {
    SCALAR_CELLS
        .iter()
        .enumerate()
        .any(|(i, c)| c.load(Ordering::Relaxed) != tuned::SCALARS[i])
        || AIR_CELLS
            .iter()
            .enumerate()
            .any(|(i, c)| c.load(Ordering::Relaxed) != tuned::AIR[i])
        || MOVE_CELLS
            .iter()
            .enumerate()
            .any(|(i, c)| c.load(Ordering::Relaxed) != tuned::MOVES[i])
        || VIEW_CELLS
            .iter()
            .enumerate()
            .any(|(i, c)| c.load(Ordering::Relaxed) != tuned::VIEW[i])
        || MONSTER_CELLS
            .iter()
            .enumerate()
            .any(|(i, c)| c.load(Ordering::Relaxed) != tuned::MONSTER[i])
}

/// A hash of every live value.
///
/// Folded into `World::checksum` so that two peers tuned differently show up as
/// a desync on the first frame, rather than as a slow divergence that reads
/// like a netcode bug.
pub fn hash() -> u64 {
    use std::hash::Hasher;
    let mut h = std::collections::hash_map::DefaultHasher::new();
    for c in SCALAR_CELLS.iter() {
        h.write_i32(c.load(Ordering::Relaxed));
    }
    for c in AIR_CELLS.iter() {
        h.write_i32(c.load(Ordering::Relaxed));
    }
    for c in MOVE_CELLS.iter() {
        h.write_i32(c.load(Ordering::Relaxed));
    }
    for c in MONSTER_CELLS.iter() {
        h.write_i32(c.load(Ordering::Relaxed));
    }
    // `VIEW_CELLS` **is** folded in, and used not to be. The camera decides
    // where the eye is, the eye is where the aiming ray starts, and so the
    // camera decides where abilities land -- see `crate::aim`. Two peers framing
    // the fight differently would place a fire pillar in different spots and
    // neither would be wrong, which is exactly what this hash exists to make
    // loud rather than quiet.
    for c in VIEW_CELLS.iter() {
        h.write_i32(c.load(Ordering::Relaxed));
    }
    h.finish()
}

// ---------------------------------------------------------------------------
// The registry the palette reads
// ---------------------------------------------------------------------------

/// One editable value, with everything the palette needs to show it.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Knob {
    Scalar(Scalar),
    Air(Class, AirField),
    Move(Class, usize, MoveField),
    Monster(usize, MonsterField),
    View(ViewKnob),
}

impl Knob {
    /// The family a knob belongs to. Families are how three hundred numbers
    /// stay navigable without search; search is how you find one when they do
    /// not.
    pub fn family(self) -> String {
        match self {
            Knob::Scalar(s) => s.family().to_string(),
            Knob::View(_) => "Camera".to_string(),
            Knob::Air(c, _) => format!("Air · {}", c.name()),
            // No "unbound slot" heading any more: storage is packed to each
            // class's own count, so a slot a class does not have is a slot
            // with no knobs rather than a row of zeroes needing an excuse.
            Knob::Move(c, slot, _) => format!(
                "{} · {} [{}]",
                c.name(),
                crate::moves::get(c, slot as u8).name,
                crate::moves::binding(c, slot)
            ),
            Knob::Monster(slot, _) => {
                format!("Ridgeback · {}", crate::monster::MOVE_NAMES[slot])
            }
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Knob::Scalar(s) => s.label(),
            Knob::View(k) => k.label(),
            Knob::Air(_, f) => f.label(),
            Knob::Move(_, _, f) => f.label(),
            Knob::Monster(_, f) => f.label(),
        }
    }

    /// A stable identifier. Used for search and for the comments in `tuned.rs`,
    /// which is what makes a baked diff readable.
    pub fn id(self) -> String {
        fn slug(s: &str) -> String {
            s.to_lowercase().replace(' ', "_")
        }
        match self {
            Knob::Scalar(s) => format!("{}.{}", slug(s.family()), slug(s.label())),
            Knob::View(k) => format!("camera.{}", slug(k.label())),
            Knob::Air(c, f) => format!("air.{}.{}", slug(c.name()), slug(f.label())),
            Knob::Move(c, slot, f) => format!(
                "move.{}.{}.{}",
                slug(c.name()),
                slug(crate::moves::get(c, slot as u8).name),
                slug(f.label())
            ),
            Knob::Monster(slot, f) => format!(
                "monster.{}.{}",
                slug(crate::monster::MOVE_NAMES[slot]),
                slug(f.label())
            ),
        }
    }

    pub fn unit(self) -> Unit {
        match self {
            Knob::Scalar(s) => s.unit(),
            Knob::View(k) => k.unit(),
            Knob::Air(_, _) => Unit::Fixed,
            Knob::Move(_, _, f) => f.unit(),
            Knob::Monster(_, f) => f.unit(),
        }
    }

    pub const fn range(self) -> (i32, i32) {
        match self {
            Knob::Scalar(s) => s.range(),
            Knob::View(k) => k.range(),
            Knob::Air(_, f) => f.range(),
            Knob::Move(_, _, f) => f.range(),
            Knob::Monster(_, f) => f.range(),
        }
    }

    pub fn raw(self) -> i32 {
        match self {
            Knob::Scalar(s) => scalar(s),
            Knob::View(k) => view(k),
            Knob::Air(c, f) => air(c, f),
            Knob::Move(c, slot, f) => move_field(c, slot, f),
            Knob::Monster(slot, f) => monster_field(slot, f),
        }
    }

    pub fn set_raw(self, raw: i32) {
        match self {
            Knob::Scalar(s) => set_scalar(s, raw),
            Knob::View(k) => set_view(k, raw),
            Knob::Air(c, f) => set_air(c, f, raw),
            Knob::Move(c, slot, f) => set_move_field(c, slot, f, raw),
            Knob::Monster(slot, f) => set_monster_field(slot, f, raw),
        }
    }

    /// What is committed in `tuned.rs`, for showing how far a knob has drifted.
    pub fn baked_raw(self) -> i32 {
        match self {
            Knob::Scalar(s) => tuned::SCALARS[s as usize],
            Knob::View(k) => tuned::VIEW[k as usize],
            Knob::Air(c, f) => tuned::AIR[air_index(c, f)],
            Knob::Move(c, slot, f) => tuned::MOVES[move_index(c, slot, f)],
            Knob::Monster(slot, f) => tuned::MONSTER[monster_index(slot, f)],
        }
    }

    pub fn is_dirty(self) -> bool {
        self.raw() != self.baked_raw()
    }
}

/// Knobs collected under their family, each family appearing once, in the order
/// families are first met.
///
/// Deliberately not "runs of adjacent knobs sharing a family". The registry's
/// order is a *storage* concern — appending a new knob to the end is what keeps
/// every existing index in `tuned.rs` valid, so a knob almost always arrives
/// away from its relatives. The first version of the palette grouped by
/// adjacency and drew a second "Air" header the moment two air scalars were
/// appended, which egui flagged as a duplicate widget.
pub fn grouped(knobs: &[Knob]) -> Vec<(String, Vec<Knob>)> {
    let mut out: Vec<(String, Vec<Knob>)> = Vec::new();
    for knob in knobs {
        let family = knob.family();
        match out.iter_mut().find(|(name, _)| *name == family) {
            Some((_, members)) => members.push(*knob),
            None => out.push((family, vec![*knob])),
        }
    }
    out
}

/// Every knob, in a stable order: universal rules, then air, then move data.
pub fn all_knobs() -> Vec<Knob> {
    let mut out: Vec<Knob> = Scalar::ALL.iter().map(|s| Knob::Scalar(*s)).collect();
    for class in ALL_CLASSES {
        for field in AirField::ALL {
            out.push(Knob::Air(class, *field));
        }
    }
    for class in ALL_CLASSES {
        for slot in 0..crate::moves::slots(class) {
            for field in MoveField::ALL {
                out.push(Knob::Move(class, slot, *field));
            }
        }
    }
    // Appended last, so every index already written into `tuned.rs` keeps the
    // meaning it was baked with. That is the rule the whole store relies on.
    for slot in 0..MONSTER_MOVES {
        for field in MonsterField::ALL {
            out.push(Knob::Monster(slot, *field));
        }
    }
    for k in ViewKnob::ALL {
        out.push(Knob::View(*k));
    }
    out
}

// ---------------------------------------------------------------------------
// Baking
// ---------------------------------------------------------------------------

/// Emit the live values as the source of `crates/sim/src/tuned.rs`.
///
/// Generated Rust rather than a data file, for the same reasons the baked
/// animations are: no loader, no asset path, no runtime parsing, and a diff
/// that shows exactly which numbers a tuning session moved. Every entry carries
/// its identifier as a comment, which is what makes that diff readable by
/// someone who was not in the session.
pub fn emit() -> String {
    let mut out = String::new();
    out.push_str(
        "//! Baked tuning values. GENERATED -- do not edit by hand.\n\
         //!\n\
         //! Written by the Oven's bake button, or by `cargo run -p sim --bin bake_tuning`.\n\
         //! Edit these in the running game (F7) and bake; the palette is the editor.\n\
         //!\n\
         //! Each entry is an integer: fixed-point values are raw 16.16 bits,\n\
         //! frame counts are frames. See `oven::Unit`.\n\n",
    );

    let knobs = all_knobs();
    let write_block = |out: &mut String, name: &str, count: usize, from: usize| {
        // `rustfmt::skip` so the file stays byte-identical to what `emit`
        // produces. Without it `cargo fmt` re-columnises the arrays, the
        // generated file stops matching its generator, and the test that
        // catches a stale bake has to be weakened to a fuzzy comparison.
        out.push_str(&format!(
            "#[rustfmt::skip]\npub const {name}: [i32; {count}] = [\n"
        ));
        for k in knobs.iter().skip(from).take(count) {
            out.push_str(&format!(
                "    {:>12}, // {} = {}\n",
                k.raw(),
                k.id(),
                k.unit().show(k.raw())
            ));
        }
        out.push_str("];\n\n");
    };

    write_block(&mut out, "SCALARS", SCALAR_COUNT, 0);
    write_block(&mut out, "AIR", AIR_COUNT, SCALAR_COUNT);
    write_block(&mut out, "MOVES", MOVE_COUNT, SCALAR_COUNT + AIR_COUNT);
    write_block(
        &mut out,
        "MONSTER",
        MONSTER_COUNT,
        SCALAR_COUNT + AIR_COUNT + MOVE_COUNT,
    );
    write_block(
        &mut out,
        "VIEW",
        VIEW_COUNT,
        SCALAR_COUNT + AIR_COUNT + MOVE_COUNT + MONSTER_COUNT,
    );
    // Exactly one trailing newline: `cargo fmt` strips a blank line at the end
    // of a file, which would leave the generated file permanently one byte
    // different from its generator.
    out.truncate(out.trim_end().len());
    out.push('\n');
    out
}
