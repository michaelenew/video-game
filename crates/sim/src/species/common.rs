//! The numbers every creature has.
//!
//! How big it is, how fast it walks and turns, how often it glances, how much
//! it takes to topple it: the machinery in `monster.rs` reads all of these, so
//! every species has a copy of each, in its own knob store and its own baked
//! file, under its own name in the Oven. The Ridgeback's are the ones these
//! were written for, and each accessor's note is about what the number did
//! there -- see `docs/design/monsters.md`.
//!
//! A species' **own** knobs -- the ones no other creature has, like the
//! Ridgeback's per-part hide and its shake force -- are declared in its own
//! file with [`species_knobs!`](crate::species_knobs), and read through
//! `Species::own_raw`.

use crate::fixed::Fx;
use crate::oven::KnobDecl;

use super::Species;

/// Raw helper for writing fixed-point bounds readably.
const fn fx(num: i32, den: i32) -> i32 {
    Fx::ratio(num, den).raw()
}

/// Declare a list of knobs: an enum naming them, and the table the Oven reads.
///
/// ```ignore
/// species_knobs! {
///     pub enum Knob;
///     Head, "hide", "Head (x damage)", Fixed, 0, fx(3, 1);
/// }
/// ```
///
/// The second column is the family *within* the species: `"hide"` shows as
/// "Ridgeback · hide" in the palette, and `""` as plain "Ridgeback".
#[macro_export]
macro_rules! species_knobs {
    (
        $(#[$meta:meta])*
        pub enum $name:ident;
        $($variant:ident, $family:literal, $label:literal, $unit:ident, $lo:expr, $hi:expr;)*
    ) => {
        $(#[$meta])*
        #[derive(Clone, Copy, PartialEq, Eq, Debug)]
        pub enum $name { $($variant,)* }

        impl $name {
            pub const ALL: &'static [$name] = &[$($name::$variant,)*];
            /// What the Oven shows and bakes, in declaration order.
            pub const DECLS: &'static [$crate::oven::KnobDecl] = &[$(
                $crate::oven::KnobDecl {
                    family: $family,
                    label: $label,
                    unit: $crate::oven::Unit::$unit,
                    lo: $lo,
                    hi: $hi,
                },
            )*];
        }
    };
}

species_knobs! {
    /// Every creature's copy of the numbers the shared machinery reads.
    pub enum Common;
    Scale,            "",      "Size (x)",                   Fixed,   fx(1,2),   fx(3,1);
    Health,           "",      "Health",                     Int,     500,       30000;
    PartHealth,       "",      "Breakable part health",      Int,     100,       6000;
    Margin,           "",      "Keep-out from the wall",     Fixed,   0,         fx(12,1);
    Walk,             "",      "Walk speed",                 Fixed,   0,         fx(20,1);
    Gallop,           "",      "Gallop speed",               Fixed,   fx(1,1),   fx(30,1);
    Back,             "",      "Backing-off speed",          Fixed,   0,         fx(12,1);
    Accel,            "",      "Walk acceleration",          Fixed,   fx(1,1),   fx(40,1);
    ProwlRange,       "",      "Preferred distance",         Fixed,   fx(1,1),   fx(20,1);
    ApproachGain,     "",      "Approach gain",              Fixed,   0,         fx(6,1);
    TurnRateMax,      "",      "Turn rate cap (turns/s)",    Fixed,   fx(1,100), fx(2,1);
    TurnGain,         "",      "Turn gain",                  Fixed,   fx(1,10),  fx(10,1);
    TurnAccel,        "",      "Turn acceleration",          Fixed,   fx(1,100), fx(10,1);
    TurnHurt,         "",      "Turn per broken leg (x)",    Fixed,   0,         fx(1,1);
    TurnSettle,       "",      "Turn bleed-off, committed",  Fixed,   0,         fx(1,1);
    GlanceFrames,     "mind",  "Frames between glances",     Frames,  1,         90;
    Lead,             "mind",  "Lead on the target (x)",     Fixed,   0,         fx(2,1);
    ProwlLead,        "mind",  "Lead horizon, prowling",     Frames,  0,         90;
    ThinkFrames,      "mind",  "Pause between moves",        Frames,  0,         120;
    Decisiveness,     "mind",  "Decisiveness (%)",           Percent, 0,         100;
    HurtAggression,   "mind",  "Aggression when wounded",    Int,     0,         400;
    VarietyPenalty,   "mind",  "Repeat penalty",             Int,     0,         2000;
    VarietyFrames,    "mind",  "Repeat penalty decay",       Frames,  0,         300;
    PoiseMax,         "",      "Poise",                      Int,     50,        5000;
    PoiseRegen,       "",      "Poise regained per frame",   Int,     0,         60;
    ToppleFrames,     "",      "Topple length",              Frames,  30,        1500;
    StumbleFrames,    "",      "Stumble length",             Frames,  20,        900;
    FlinchFrames,     "",      "Flinch length",              Frames,  1,         90;
    FlinchThreshold,  "",      "Damage that flinches it",    Int,     1,         2000;
    LegDrop,          "legs",  "Corner drop per break",      Fixed,   0,         fx(2,1);
    LegPitch,         "legs",  "Pitch per break",            Fixed,   0,         fx(1,8);
    LegRoll,          "legs",  "Roll per break",             Fixed,   0,         fx(1,8);
    LegFold,          "legs",  "Broken leg, knee fold",      Fixed,   0,         fx(1,2);
    LegBuckle,        "legs",  "Broken leg, hip share (x)",  Fixed,   0,         fx(2,1);
    LegSpeedHurt,     "legs",  "Speed per break (x)",        Fixed,   0,         fx(1,1);
    StrainDecay,      "nerve", "Strain bled per frame (%)",  Percent, 1,         50;
    CcStrain,         "nerve", "Strain to feel control",     Int,     50,        6000;
    InterruptStrain,  "nerve", "Strain to interrupt",        Int,     50,        9000;
    StrainDesperation,"nerve", "Thresholds fall by (%)",     Percent, 0,         95;
    CcSlowBite,       "nerve", "Slow it actually feels (x)", Fixed,   0,         fx(1,1);
    CcRoot,           "nerve", "Root, per grab frame (x)",   Fixed,   0,         fx(2,1);
    CcStumble,        "nerve", "Stumble per launch (frames/mps)", Fixed, 0,      fx(30,1);
    GaitStride,       "pose",  "Stride length",              Fixed,   fx(1,2),   fx(12,1);
    BreathRate,       "pose",  "Breath per frame",           Int,     1,         2000;
    HeadTrack,        "pose",  "Head tracking (turns)",      Fixed,   0,         fx(1,4);
    Spawn,            "",      "Spawns this far out",        Fixed,   0,         fx(14,1);
    HunterSpawn,      "",      "Hunters start this far out", Fixed,   0,         fx(14,1);
    StartupTracking,  "mind",  "Windup follows you (x turn rate)", Fixed, 0,     fx(1,1);
    ClosingSpeed,     "mind",  "Closing speed that provokes it",   Fixed, 0,     fx(12,1);
    ClosingAppetite,  "mind",  "Appetite for a closing target",    Int,   0,     4000;
    ComboAppetite,    "mind",  "Appetite for a stunned target",    Int,   0,     4000;
    TargetSwitch,     "mind",  "Switch targets when nearer than (x)", Fixed, 0,  fx(1,1);
    PursuitGain,      "",      "Matches a fleeing target's speed (x)", Fixed, 0, fx(3,1);
    RearPause,        "mind",  "Turns to face you after a rear move", Frames, 0, 180;
    Brake,            "",      "Braking, committed (m/s2)",  Fixed,   fx(1,1),   fx(200,1);
    Launch,           "",      "Launching into a charge (m/s2)", Fixed, fx(10,1), fx(2000,1);
    HuntGrace,        "mind",  "Holds off when a hunt begins", Frames, 0,        600;
}

/// A knob declaration's family as the palette shows it for one species.
pub fn family(species: &Species, decl: &KnobDecl) -> String {
    if decl.family.is_empty() {
        species.name.to_string()
    } else {
        format!("{} · {}", species.name, decl.family)
    }
}

// ---------------------------------------------------------------------------
// The accessors, and what each number is for
//
// Five groups, edited separately because they answer different questions: how
// the animal is built, how it decides, what its hide is worth, what it takes
// to move it around, and what it takes to stay on its back.
// ---------------------------------------------------------------------------

impl Species {
    fn fx(&self, k: Common) -> Fx {
        Fx::from_raw(self.common(k))
    }
    fn frames(&self, k: Common) -> u16 {
        self.common(k) as u16
    }

    /// Multiplies every bone offset and every part box. The one number you
    /// actually reach for while playing -- *how big is it* -- which is why the
    /// proportions in a species' bones and parts are constants and this is
    /// not.
    ///
    /// At one the Ridgeback is about thirteen metres nose to tail with its back
    /// five and a half metres up. That height is chosen against the jump: the
    /// lowest thing on the standing animal is out of every class's hop but the
    /// Dual mage's, and the ways up are the ones the ground game earns. **The
    /// climb is a positioning problem before it is a timing one**, and this is
    /// the number that decides it. See `cargo run -p sim --bin beastcheck`.
    pub fn scale(&self) -> Fx {
        self.fx(Common::Scale)
    }

    /// Pool health. The per-part vulnerabilities are what decide how long a
    /// fight actually takes, so this is the *second* number to reach for.
    pub fn health(&self) -> i32 {
        self.common(Common::Health)
    }

    /// Health of one breakable part. On the Ridgeback, a foot: breaking one
    /// drops that corner of the animal for good and puts it on its knee for a
    /// moment, which is the ground game's whole payout -- so this is the
    /// number that says how long the ground game is.
    pub fn part_health(&self) -> i32 {
        self.common(Common::PartHealth)
    }

    /// How far from the arena wall it is kept. Thirteen metres of animal in a
    /// twenty-eight metre arena needs somewhere to stand.
    pub fn margin(&self) -> Fx {
        self.fx(Common::Margin)
    }

    /// Walking, backing off, and how quickly it changes between them. The walk
    /// is below the player's own on purpose: inside striking distance it
    /// catches you by cornering you, not by outrunning you. The walk is also
    /// where the gait has fully changed over from standing; above it, it
    /// blends to the gallop.
    pub fn walk(&self) -> Fx {
        self.fx(Common::Walk)
    }
    pub fn back(&self) -> Fx {
        self.fx(Common::Back)
    }
    pub fn accel(&self) -> Fx {
        self.fx(Common::Accel)
    }

    /// **It plants its feet for a move.** How hard it stops once committed to
    /// anything that does not itself travel. At the walking acceleration a
    /// creature that had been galloping slid most of a body length through a
    /// stomp's windup, and a stomp is faster than anybody reacts -- so it hit
    /// people from outside its own reach, which is the one kind of hit the
    /// fight report calls unanswerable.
    pub fn brake(&self) -> Fx {
        self.fx(Common::Brake)
    }

    /// **How hard it launches into a move that travels.** The charge and the
    /// bite's lunge reach their speed in a few frames rather than one: a single
    /// frame is an unbounded acceleration, and the grip test that decides
    /// whether a rider holds on reads acceleration -- so a charge from a
    /// standstill threw braced riders off the barrel, which the design says
    /// only the slam does. Under twice the grip, braced; over it, loose.
    pub fn launch(&self) -> Fx {
        self.fx(Common::Launch)
    }

    /// The speed the gait has fully changed over to a gallop at, **and the
    /// fastest it goes**: wanting to close a long gap runs it up to this. Above
    /// the player's walk on purpose, since 2026-09-23 -- a fight you could walk
    /// away from at leisure was not a hunt. Between this and the walk the two
    /// cycles are blended, which is what stops a creature accelerating out of
    /// a walk from planting two feet at once.
    pub fn gallop(&self) -> Fx {
        self.fx(Common::Gallop)
    }

    /// The distance it tries to hold, and how hard it corrects toward it. Set
    /// near the middle of the move set's range band so that most of what it
    /// wants to do is available most of the time. The correction is scaled by
    /// how squarely it faces the target -- it turns before it runs -- see
    /// `Monster::walk`.
    pub fn prowl_range(&self) -> Fx {
        self.fx(Common::ProwlRange)
    }
    pub fn approach_gain(&self) -> Fx {
        self.fx(Common::ApproachGain)
    }
    /// **A fleeing target is a galloped-at target.** The speed the target is
    /// moving away at is added to what the distance alone asks for, times
    /// this, so backpedalling from just outside its reach is not a way to stay
    /// there. See `monster::Monster::walk`.
    pub fn pursuit_gain(&self) -> Fx {
        self.fx(Common::PursuitGain)
    }

    /// Turning, in turns per second and turns per second squared.
    ///
    /// The rate cap is how fast it can come round. The acceleration cap is what
    /// gives it mass -- and it is the more interesting of the two, because it
    /// is what produces the overshoot a player cuts back across. See
    /// `monster::Monster::steer`.
    pub fn turn_rate_max(&self) -> Fx {
        self.fx(Common::TurnRateMax)
    }
    pub fn turn_gain(&self) -> Fx {
        self.fx(Common::TurnGain)
    }
    pub fn turn_accel(&self) -> Fx {
        self.fx(Common::TurnAccel)
    }
    /// What each broken leg does to the turn toward that side. Applied once
    /// per break, so an animal with both legs gone on one side barely comes
    /// round at all -- which is a thing the player did, and can see.
    pub fn turn_hurt(&self) -> Fx {
        self.fx(Common::TurnHurt)
    }
    /// How a swing in progress bleeds off once a move commits. Stopping dead
    /// visibly clamps the animal mid-turn.
    pub fn turn_settle(&self) -> Fx {
        self.fx(Common::TurnSettle)
    }

    /// **The difficulty model, both halves.**
    ///
    /// `glance_frames` is how stale its information is: it takes one sample of
    /// the target and works from it until the next. `lead` is how much of the
    /// extrapolation from that sample it trusts, as a fraction of the move's
    /// own startup -- so a slow lunge leads further, which is both correct and
    /// one knob instead of one per move.
    ///
    /// Short glance with a long lead is frightening. Long glance with no lead
    /// is an animal you can walk around. The skill the pair rewards is
    /// specific: change direction between its glances.
    pub fn glance_frames(&self) -> u16 {
        self.frames(Common::GlanceFrames)
    }
    pub fn lead(&self) -> Fx {
        self.fx(Common::Lead)
    }
    /// The horizon it leads by while merely walking toward you, in frames.
    pub fn prowl_lead(&self) -> u16 {
        self.frames(Common::ProwlLead)
    }

    /// The pause between moves. Without one it is a chain gun.
    pub fn think_frames(&self) -> u16 {
        self.frames(Common::ThinkFrames)
    }

    /// How close to the best a move has to score to make the draw, as a
    /// percentage. A hundred always throws the best move and is therefore a
    /// script you can memorise; zero is noise. In between, the distribution is
    /// learnable and the next move is not.
    pub fn decisiveness(&self) -> i32 {
        self.common(Common::Decisiveness)
    }

    /// How much harder it commits as it loses health, and how much it dislikes
    /// repeating itself and for how long.
    pub fn hurt_aggression(&self) -> i32 {
        self.common(Common::HurtAggression)
    }
    pub fn variety_penalty(&self) -> i32 {
        self.common(Common::VarietyPenalty)
    }
    pub fn variety_frames(&self) -> u16 {
        self.frames(Common::VarietyFrames)
    }

    /// **The windup follows you.** How much of its free turn rate it keeps
    /// during a startup, before the yaw locks on the first active frame. Zero
    /// is the old rule -- facing locked at commit -- and one is a tell you
    /// cannot walk out of at all. See `monster::Monster::steer`.
    pub fn startup_tracking(&self) -> Fx {
        self.fx(Common::StartupTracking)
    }

    /// **Walking up to it is provoked.** A target closing faster than this
    /// raises its appetite for every forward move that fits by
    /// `closing_appetite`, so an approach is met rather than watched.
    pub fn closing_speed(&self) -> Fx {
        self.fx(Common::ClosingSpeed)
    }
    pub fn closing_appetite(&self) -> i32 {
        self.common(Common::ClosingAppetite)
    }

    /// **A stunned target is a target to follow up on.** Added to every forward
    /// move's appetite while the target cannot act: the sweep's stagger and the
    /// spray's root are set-ups, and this is what makes it press them.
    pub fn combo_appetite(&self) -> i32 {
        self.common(Common::ComboAppetite)
    }

    /// **After a move aimed behind it, it comes about.** The pause before its
    /// next move, in place of `think_frames`, after a move aimed behind it
    /// (the Ridgeback's sweep and kick). Without it a target standing behind
    /// the animal is a target only the rear moves can score against, so it
    /// threw them one after the other for ten minutes and never turned round
    /// -- which is a turret, not an animal. The pause is spent turning at the
    /// full rate, and it is a real opening at the rear: the walk-up window
    /// behind the hips. See `monster::Monster::tick_action`.
    pub fn rear_pause(&self) -> u16 {
        self.frames(Common::RearPause)
    }

    /// **A moment to get your bearings.** Frames at the start of a hunt in
    /// which it is taking you in: it stands its ground and turns to face you,
    /// and throws nothing. Hitting it ends the moment at once. Without it the
    /// spray and the charge landed before a player had moved the camera.
    ///
    /// Standing rather than closing, because the first version walked toward
    /// you through it -- and a fighter who spent the moment getting their
    /// bearings was standing under it when the moment ended, which only moved
    /// the ambush three seconds later.
    pub fn hunt_grace(&self) -> u16 {
        self.frames(Common::HuntGrace)
    }

    /// **It keeps the target it has.** Another fighter has to be nearer than
    /// this fraction of the current target's distance before its attention
    /// moves.
    pub fn target_switch(&self) -> Fx {
        self.fx(Common::TargetSwitch)
    }

    /// Damage to a weak point fills the poise pool; a full pool is a topple.
    /// Regeneration is what stops a topple being saved up across a whole
    /// fight.
    pub fn poise_max(&self) -> i32 {
        self.common(Common::PoiseMax)
    }
    pub fn poise_regen(&self) -> i32 {
        self.common(Common::PoiseRegen)
    }
    pub fn topple_frames(&self) -> u16 {
        self.frames(Common::ToppleFrames)
    }

    /// Down on a knee: the middle rung of the ladder, and the one the ground
    /// game aims at. Long enough to cross the ground and climb on -- that is
    /// what the number is for, so it is set against the run-up rather than
    /// against a feeling.
    pub fn stumble_frames(&self) -> u16 {
        self.frames(Common::StumbleFrames)
    }

    /// The cheap reaction. It interrupts a startup or a recovery but never an
    /// active frame, and only a hit that hurts causes one -- which in practice
    /// means a committed move rather than an auto.
    pub fn flinch_frames(&self) -> u16 {
        self.frames(Common::FlinchFrames)
    }
    pub fn flinch_threshold(&self) -> i32 {
        self.common(Common::FlinchThreshold)
    }

    /// What a broken leg leaves behind, once the stumble is over.
    ///
    /// The corner drops, the body pitches toward the missing end, rolls toward
    /// the missing side, and the useless limb folds rather than punching
    /// through the floor. The drop is the one that matters for the fight:
    /// enough of them and the back comes within reach of a standing jump.
    pub fn leg_drop(&self) -> Fx {
        self.fx(Common::LegDrop)
    }
    pub fn leg_pitch(&self) -> Fx {
        self.fx(Common::LegPitch)
    }
    pub fn leg_roll(&self) -> Fx {
        self.fx(Common::LegRoll)
    }
    pub fn leg_fold(&self) -> Fx {
        self.fx(Common::LegFold)
    }
    /// How much of the fold the joint *above* the break takes. A limb that
    /// folds only at its lower joint sticks out sideways; one that folds at
    /// both tucks under the body, which is what a broken leg does.
    pub fn leg_buckle(&self) -> Fx {
        self.fx(Common::LegBuckle)
    }
    /// Speed lost per broken leg, as a multiplier applied once per break.
    pub fn leg_speed_hurt(&self) -> Fx {
        self.fx(Common::LegSpeedHurt)
    }

    /// **The crowd-control model.** See `docs/design/monsters.md` §4.
    ///
    /// `strain` is damage taken recently: every hit adds to it and
    /// `strain_decay` per cent of it bleeds away every frame, so a burst fills
    /// it and a trickle does not. Above `cc_strain` the creature is susceptible
    /// to control at all; above `interrupt_strain` a hit breaks it out of
    /// whatever it is doing, live hitbox included.
    ///
    /// `strain_desperation` is how far both bars fall by the time it is nearly
    /// dead. That one number is the arc of a hunt: methodical while the animal
    /// is fresh, frantic once it is not.
    pub fn strain_decay(&self) -> i32 {
        self.common(Common::StrainDecay)
    }
    pub fn cc_strain(&self) -> i32 {
        self.common(Common::CcStrain)
    }
    pub fn interrupt_strain(&self) -> i32 {
        self.common(Common::InterruptStrain)
    }
    pub fn strain_desperation(&self) -> i32 {
        self.common(Common::StrainDesperation)
    }

    /// How much of a slow it actually feels, once it is susceptible. Zero means
    /// a slow never touches it; one means it takes the same slow a fighter
    /// does.
    pub fn cc_slow_bite(&self) -> Fx {
        self.fx(Common::CcSlowBite)
    }
    /// Frames rooted per frame the grab would have held a fighter.
    pub fn cc_root(&self) -> Fx {
        self.fx(Common::CcRoot)
    }
    /// Frames of stumble per metre per second of launch. A knock-up on
    /// something this heavy is a trip rather than a lift, and this is the
    /// exchange rate.
    pub fn cc_stumble(&self) -> Fx {
        self.fx(Common::CcStumble)
    }

    /// Metres per cycle of the gait, and how fast it breathes when standing.
    ///
    /// The gait is indexed by ground covered rather than by time, so this is
    /// what decides where a footfall lands. The same rule the fighters'
    /// locomotion follows, for the same reason: a cycle on a fixed cadence
    /// skates.
    pub fn gait_stride(&self) -> Fx {
        self.fx(Common::GaitStride)
    }
    pub fn breath_rate(&self) -> u16 {
        self.frames(Common::BreathRate)
    }

    /// How far round the neck will turn to keep you in view, in turns. Split
    /// across the neck's bones, so it curves rather than hinging.
    pub fn head_track(&self) -> Fx {
        self.fx(Common::HeadTrack)
    }

    /// Where the creature stands when a hunt begins, and where the hunters do.
    pub fn spawn(&self) -> Fx {
        self.fx(Common::Spawn)
    }
    pub fn hunter_spawn(&self) -> Fx {
        self.fx(Common::HunterSpawn)
    }
}
