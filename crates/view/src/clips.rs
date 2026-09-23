//! The list of animations the game knows how to play.
//!
//! This file is a **contract, not content**. It names every clip, says how long
//! it is, which family it belongs to and what it is supposed to communicate;
//! the poses that fill it live in `crates/anim/src/clips/`. Keeping the two
//! apart is what lets the animation for a dozen unrelated moves be worked on at
//! the same time without anybody colliding: the names are fixed here first, and
//! the bake refuses to run if any of them is missing.
//!
//! It is also the answer to "what is missing". A clip with no recipe is a
//! compile-time error rather than a character who silently T-poses in a corner
//! of a match nobody was watching.
//!
//! ## Lengths
//!
//! Attack clips take their length from the **move table**, not from a number
//! typed here, so retuning a move's startup in the Oven and re-baking moves the
//! animation with it. Everything else is a fixed count chosen for the motion.

use sim::Class;

/// Which broad thing an animation is for. Families group the hub's clip list
/// and decide which file a clip is authored in.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Family {
    Locomotion,
    Turns,
    Air,
    Dodge,
    Defence,
    Reactions,
    Moves,
}

impl Family {
    pub const fn name(self) -> &'static str {
        match self {
            Family::Locomotion => "locomotion",
            Family::Turns => "turns and crouch",
            Family::Air => "air",
            Family::Dodge => "dodge",
            Family::Defence => "defence",
            Family::Reactions => "reactions",
            Family::Moves => "moves",
        }
    }
}

macro_rules! clips {
    ($($variant:ident, $name:literal, $family:ident, $file:literal, $len:expr, $loops:literal, $what:literal;)*) => {
        /// Every animation in the game, by name.
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
        pub enum Clip { $($variant,)* }

        pub const ALL: &[Clip] = &[$(Clip::$variant,)*];
        pub const CLIP_COUNT: usize = ALL.len();

        impl Clip {
            /// Snake case, and the same string the recipe uses. The bake pairs
            /// them up by this.
            pub const fn name(self) -> &'static str {
                match self { $(Clip::$variant => $name,)* }
            }

            pub const fn family(self) -> Family {
                match self { $(Clip::$variant => Family::$family,)* }
            }

            /// Which file in `crates/anim/src/clips/` holds the recipe. The hub
            /// writes edits back to it.
            pub const fn file(self) -> &'static str {
                match self { $(Clip::$variant => $file,)* }
            }

            pub const fn looping(self) -> bool {
                match self { $(Clip::$variant => $loops,)* }
            }

            /// What this clip has to communicate, in one line. Read by whoever
            /// authors it, and shown in the hub beside the timeline -- an
            /// animation that does not say what it is for gets tuned into
            /// something pretty that reads wrong.
            pub const fn what(self) -> &'static str {
                match self { $(Clip::$variant => $what,)* }
            }

            fn length_spec(self) -> Length {
                match self { $(Clip::$variant => $len,)* }
            }
        }
    };
}

/// Where a clip's frame count comes from.
#[derive(Clone, Copy, Debug)]
pub enum Length {
    Fixed(u16),
    /// Startup plus active plus recovery of one class's move, live from the
    /// Oven. An attack animation that disagrees with its own frame data is
    /// worse than no animation: it teaches the opponent the wrong timing.
    Move(Class, u8),
    Dodge,
    AirDodge,
    /// The Reaver's shadow, in flight. The throw is a simulation clock, so a
    /// clip of any other length would either be cut off or hold its last frame
    /// while the body kept travelling.
    ShadowFlight,
}

clips! {
    // -- Locomotion ---------------------------------------------------------
    Idle,            "idle",            Locomotion, "locomotion", Length::Fixed(120), true,
        "Standing, alive, not doing anything. Breathing and a slow weight shift -- the pose a player stares at most.";
    WalkForward,     "walk_forward",    Locomotion, "locomotion", Length::Fixed(34), true,
        "A measured advance. Contact, weight over the foot, push off; arms counter-swing.";
    WalkBack,        "walk_back",       Locomotion, "locomotion", Length::Fixed(38), true,
        "Giving ground without turning. Weight stays back, steps are shorter, and the guard side leads.";
    WalkLeft,        "walk_left",       Locomotion, "locomotion", Length::Fixed(34), true,
        "Sidestep left, facing forward. Feet cross under, never over -- crossing over reads as losing balance.";
    WalkRight,       "walk_right",      Locomotion, "locomotion", Length::Fixed(34), true,
        "Sidestep right, facing forward.";
    RunForward,      "run_forward",     Locomotion, "locomotion", Length::Fixed(22), true,
        "Committed sprint. Longer stride, deeper lean, a flight phase where neither foot is down.";
    RunBack,         "run_back",        Locomotion, "locomotion", Length::Fixed(26), true,
        "Backpedal at speed. Reads as retreating, not as a walk played backwards.";
    RunLeft,         "run_left",        Locomotion, "locomotion", Length::Fixed(24), true,
        "Hard sidestep left, shoulders squared to the front.";
    RunRight,        "run_right",       Locomotion, "locomotion", Length::Fixed(24), true,
        "Hard sidestep right.";

    // -- Turning and crouching ---------------------------------------------
    TurnLeftSlow,    "turn_left_slow",  Turns, "turns", Length::Fixed(24), false,
        "Pivoting left while the mouse turns gently. Lean into the turn, feet shuffle under the hips.";
    TurnRightSlow,   "turn_right_slow", Turns, "turns", Length::Fixed(24), false,
        "The same to the right.";
    TurnLeftFast,    "turn_left_fast",  Turns, "turns", Length::Fixed(14), false,
        "A whipped turn: the head leads, the shoulders follow, the feet cross and plant.";
    TurnRightFast,   "turn_right_fast", Turns, "turns", Length::Fixed(14), false,
        "The same to the right.";
    CrouchIn,        "crouch_in",       Turns, "turns", Length::Fixed(8), false,
        "Dropping into a crouch. Has to read instantly -- the attacker needs to know their overhead will whiff.";
    CrouchIdle,      "crouch_idle",     Turns, "turns", Length::Fixed(96), true,
        "Held low. Compact, coiled, unmistakably shorter than standing.";
    CrouchWalk,      "crouch_walk",     Turns, "turns", Length::Fixed(44), true,
        "Shuffling while crouched. Slow and deliberate; the head must not bob back up.";

    // -- Air ---------------------------------------------------------------
    JumpTakeoff,     "jump_takeoff",    Air, "air", Length::Fixed(6), false,
        "The coil and push. Six frames to sell the whole jump, so the crouch is deep and the extension snaps.";
    JumpRise,        "jump_rise",       Air, "air", Length::Fixed(18), true,
        "Going up. Legs trail, arms up, body long. Held for as long as the rise lasts.";
    JumpApex,        "jump_apex",       Air, "air", Length::Fixed(14), false,
        "The turn at the top. The one place in a jump where a player can read how much height they got.";
    JumpFall,        "jump_fall",       Air, "air", Length::Fixed(20), true,
        "Descending, legs gathering under for the landing.";
    LandSoft,        "land_soft",       Air, "air", Length::Fixed(12), false,
        "Absorbing a short drop. Knees give a little and recover.";
    LandHeavy,       "land_heavy",      Air, "air", Length::Fixed(20), false,
        "Absorbing a real fall. Deep, with a hand down and a slow recovery.";

    // -- Dodging -----------------------------------------------------------
    DodgeForward,    "dodge_forward",   Dodge, "dodge", Length::Dodge, false,
        "A committed dive forward. Invulnerable early, vulnerable late -- and the two halves must look different.";
    DodgeBack,       "dodge_back",      Dodge, "dodge", Length::Dodge, false,
        "Hop backwards out of range. Reads as evasive from across the arena.";
    DodgeLeft,       "dodge_left",      Dodge, "dodge", Length::Dodge, false,
        "Roll left. Low and tucked through the invulnerable frames, then rising.";
    DodgeRight,      "dodge_right",     Dodge, "dodge", Length::Dodge, false,
        "Roll right.";
    AirDodge,        "air_dodge",       Dodge, "dodge", Length::AirDodge, false,
        "A sideways commitment in mid-air, once per jump. Should read as a decision, not as flight.";

    // -- Defence -----------------------------------------------------------
    GuardIn,         "guard_in",        Defence, "defence", Length::Fixed(8), false,
        "Coming up to guard. Deliberate and quick: the parry window is the first four frames.";
    GuardIdle,       "guard_idle",      Defence, "defence", Length::Fixed(96), true,
        "Held guard. Side-on, weight even, small live adjustments -- not a statue.";
    GuardOut,        "guard_out",       Defence, "defence", Length::Fixed(8), false,
        "Dropping guard back to neutral.";
    Parry,           "parry",           Defence, "defence", Length::Fixed(14), false,
        "The guard that caught it. A sharp deflecting turn -- the defender earned this and should see it.";
    BlockStun,       "block_stun",      Defence, "defence", Length::Fixed(14), false,
        "Absorbing a blocked hit. Shoved back on the heels, guard intact.";

    // -- Getting hit -------------------------------------------------------
    HitLight,        "hit_light",       Reactions, "reactions", Length::Fixed(16), false,
        "A quick hit. Head snaps, torso folds a little, recovers fast.";
    HitHeavy,        "hit_heavy",       Reactions, "reactions", Length::Fixed(26), false,
        "A committed hit landing. Nothing about this should look controlled.";
    Stagger,         "stagger",         Reactions, "reactions", Length::Fixed(34), false,
        "Off balance and open. The punish window, and it should look like one.";
    Launched,        "launched",        Reactions, "reactions", Length::Fixed(24), true,
        "Airborne against your will, after an uppercut. Limbs trailing, no control.";
    Grabbed,         "grabbed",         Reactions, "reactions", Length::Fixed(30), true,
        "Held at arm's length by someone else. Continuous and struggling, never relaxing.";
    Defeat,          "defeat",          Reactions, "reactions", Length::Fixed(52), false,
        "Out of health. Collapse, and stay down.";

    // -- The Bulwark: Bash, Slam, Grapple ----------------------------------
    BulwarkPoke,     "bulwark_poke",      Moves, "bulwark", Length::Move(Class::Bulwark, 0), false,
        "Bash: a short shield punch. Fast, minus on block, thrown constantly -- it must not root the silhouette.";
    BulwarkCommitted,"bulwark_committed", Moves, "bulwark", Length::Move(Class::Bulwark, 1), false,
        "Slam: the overhead. Heavy, telegraphed on purpose, and the telegraph has to be legible from frame two.";
    BulwarkSpecial,  "bulwark_special",   Moves, "bulwark", Length::Move(Class::Bulwark, 2), false,
        "Grapple: beats guard outright, loses to dodge. A committed forward reach that ends holding someone.";

    // -- The Champion: three weapons on three buttons -----------------------
    //
    // Nineteen clips, laid out the way the move list is: a row per situation
    // and a column per weapon. Read down a column and it is one weapon in six
    // situations, so the grip and the weight have to stay recognisable. Read
    // across a row and it is three weapons in one situation, so the shape has
    // to differ completely. Both readings have to hold in the silhouette, which
    // is what these are for.
    //
    // The first three rows are **one chain**, three hits deep, and every hit is
    // a free choice of all three weapons -- so a clip's job is doubled. It has
    // to say which weapon, and it has to say how deep into a string you are,
    // because the opponent's read is "can I still block the next one" and the
    // answer is written in the body rather than on the HUD. The openers leave
    // the guard and come back to it; the second hits carry the follow-through
    // of whatever came before straight into the next swing; the finishers
    // commit the feet.
    ChampionSword,    "champion_sword",     Moves, "champion", Length::Move(Class::Champion, 0), false,
        "Sword, first hit: a descending diagonal off the right shoulder, thrown on a step. Corner to corner, not across the front.";
    ChampionHammer,   "champion_hammer",    Moves, "champion", Length::Move(Class::Champion, 1), false,
        "Hammer, first hit: overhead to the floor. All of the weight above the head, then all of it on the ground.";
    ChampionSpear,    "champion_spear",     Moves, "champion", Length::Move(Class::Champion, 2), false,
        "Spear, first hit: a one-armed jab off the leading hand, butt of the shaft still at the hip. Feet still.";
    ChampionBackcut,  "champion_backcut",   Moves, "champion", Length::Move(Class::Champion, 3), false,
        "Sword, second hit: the same diagonal off the other shoulder, a hand higher. Never opens from guard -- it starts where the first cut ended.";
    ChampionUproot,   "champion_uproot",    Moves, "champion", Length::Move(Class::Champion, 4), false,
        "Hammer, second hit: the head torn back out of the floor and up through them. Rising, and it has to read as the reverse of the slam.";
    ChampionSkewer,   "champion_skewer",    Moves, "champion", Length::Move(Class::Champion, 5), false,
        "Spear, second hit: both hands on the haft, a long wind-up, then a dash behind the point. The heaviest stagger in the kit.";
    ChampionUpcut,    "champion_upcut",     Moves, "champion", Length::Move(Class::Champion, 6), false,
        "Sword, finisher: a rising cut out of a short dash, up the line the second one came down. The feet leave, which is the tell.";
    ChampionEarthbreaker,"champion_earthbreaker",Moves,"champion",Length::Move(Class::Champion, 7), false,
        "Hammer, finisher: a dash, the whole body over the head, and into the floor. The guard break, and it throws them up -- press jump and you go with them.";
    ChampionWhirl,    "champion_whirl",     Moves, "champion", Length::Move(Class::Champion, 8), false,
        "Spear, finisher: the shaft swept flat all the way round the body, low. The one move that clears the ring.";
    ChampionAirSword, "champion_air_sword", Moves, "champion", Length::Move(Class::Champion, 9), false,
        "Air sword: the same cut rolled into the vertical -- down across the body, not across the front of it.";
    ChampionAirHammer,"champion_air_hammer",Moves, "champion", Length::Move(Class::Champion, 10), false,
        "Air hammer: a long wind-up overhead and a smash straight down. The startup is the move; it has to read from across the arena.";
    ChampionAirSpear, "champion_air_spear", Moves, "champion", Length::Move(Class::Champion, 11), false,
        "Air spear: a wide fan swept around the aim, left to right, one-handed and light.";
    ChampionRushSlash,"champion_rush_slash",Moves, "champion", Length::Move(Class::Champion, 12), false,
        "Rush slash: cutting as you run past. Repeating figure-of-eight, the body never stopping.";
    ChampionRushSweep,"champion_rush_sweep",Moves, "champion", Length::Move(Class::Champion, 13), false,
        "Rush sweep: the hammer dragged down through the floor as you run past, low enough to take the legs. The run never stops.";
    ChampionRushStab, "champion_rush_stab", Moves, "champion", Length::Move(Class::Champion, 14), false,
        "Rush stab: the dash stopped dead and put into the point. Braced, both feet planted, everything forward.";
    ChampionRisingCut,"champion_rising_cut",Moves, "champion", Length::Move(Class::Champion, 15), false,
        "Rising cut: an angled slash up and forward that takes you off the floor with it. One line from the back foot to the point, and no hand free.";
    ChampionUppercut, "champion_uppercut",  Moves, "champion", Length::Move(Class::Champion, 16), false,
        "Uppercut: leaps, and takes whoever it catches into the air. Rising, and it leaves the ground.";
    ChampionPoleDrive,"champion_pole_drive",Moves, "champion", Length::Move(Class::Champion, 17), false,
        "Pole drive: the butt of the spear cracked into the floor at your own feet, and the ground throws you off it. A launch, not a swing.";
    ChampionVault,    "champion_vault",     Moves, "champion", Length::Move(Class::Champion, 18), false,
        "Pole vault: the spear planted in the floor and the body swung up over it. No strike in it at all.";

    // -- The Shadow Reaver: Slash, Executioner, Guillotine ------------------
    ReaverPoke,      "reaver_poke",       Moves, "reaver", Length::Move(Class::ShadowReaver, 0), false,
        "Slash: a fast diagonal cut from a light, mobile stance.";
    ReaverCommitted, "reaver_committed",  Moves, "reaver", Length::Move(Class::ShadowReaver, 1), false,
        "Executioner: a two-handed descending cut with the whole body dropped behind it.";
    ReaverSpecial,   "reaver_special",    Moves, "reaver", Length::Move(Class::ShadowReaver, 2), false,
        "Guillotine: blades erupt from the placed shadow. The caster points and commits; the violence is elsewhere.";
    ReaverMechanic,  "reaver_mechanic",   Moves, "reaver", Length::Move(Class::ShadowReaver, 3), false,
        "Send shadow: an open-handed throw that puts the second body somewhere else. Nothing leaves her hand -- what leaves is her.";

    // -- The Reaver's shadow, which is a body of its own --------------------
    //
    // Not moves: these two are what the *shadow* does with itself, and they
    // play on the second skeleton rather than on hers. Everything else it does
    // is her own animation, replayed a few frames late.
    ShadowDash,      "shadow_dash",       Moves, "reaver", Length::ShadowFlight, false,
        "The shadow travelling: thrown forward, right leg trailing at forty-five degrees, left knee up and folded, left elbow across the face, right arm streaming behind.";
    ShadowReady,     "shadow_ready",      Moves, "reaver", Length::Fixed(90), true,
        "The shadow arrived: a low bladed guard, weight back, both hands up. Waiting, and plainly still a threat.";

    // -- The Elementalist: Bolt, Fissure, Fire pillar, Cataclysm ------------
    //
    // Four rather than three: right click is otherwise dead weight on a class
    // with no shield, so Cataclysm takes it. See `moves::NAMES`.
    ElementalistPoke,     "elementalist_poke",      Moves, "elementalist", Length::Move(Class::Elementalist, 0), false,
        "Bolt: a flicked ranged jab. Quick, from the wrist and forearm.";
    ElementalistCommitted,"elementalist_committed", Moves, "elementalist", Length::Move(Class::Elementalist, 1), false,
        "Fissure: both hands driven into the ground. The power goes down, not out.";
    ElementalistSpecial,  "elementalist_special",   Moves, "elementalist", Length::Move(Class::Elementalist, 2), false,
        "Fire pillar: a rising two-handed gesture that pulls a column up out of the floor.";
    ElementalistHeavy,    "elementalist_heavy",     Moves, "elementalist", Length::Move(Class::Elementalist, 3), false,
        "Cataclysm: both hands wound up wide and thrown forward together, then let fall.";

    // -- The Blood mage: Bloodletter, Reap, Grasp, Black spike, Reaping sweep
    //
    // Five rather than three: `E` is an ability on this class, because its
    // mechanic is health and there is no state to toggle, and the auto is the
    // fifth row because it was appended when the scythe arrived. See
    // `moves::blood`. The scythe itself is not in any clip -- the two-handed
    // moves put the hands on a haft, and `crate::scythe` hangs the blade off
    // the grip at the reach the hit test uses.
    BloodPoke,      "blood_poke",       Moves, "blood", Length::Move(Class::BloodMage, 0), false,
        "Bloodletter: a blade thrown out underarm and caught again on the way back.";
    BloodCommitted, "blood_committed",  Moves, "blood", Length::Move(Class::BloodMage, 1), false,
        "Haemorrhage: one hand drawn in to the chest and thrust out flat, palm forward. The bolt leaves on the thrust.";
    BloodSpecial,   "blood_special",    Moves, "blood", Length::Move(Class::BloodMage, 2), false,
        "Grasp: both arms thrown open and snapped shut. Four arms leave wide and arrive together.";
    BloodMechanic,  "blood_mechanic",   Moves, "blood", Length::Move(Class::BloodMage, 3), false,
        "Black spike: a downward stabbing gesture that plants something in the ground.";
    BloodSweep,     "blood_sweep",      Moves, "blood", Length::Move(Class::BloodMage, 4), false,
        "Reaping sweep: both hands on the haft, gathered off the left hip and swept across the front, low to high.";

    // -- The Dual mage: two autos, two Lances, Judgement, Sweep -------------
    //
    // Six, and they are three pairs and a finisher. The two autos are the same
    // punch on opposite arms: left is dark, right is light, and which one just
    // landed is how the class steers its meter. They are two clips rather than
    // one played twice because a mirrored pose is a mirrored *pose* -- see
    // `Pose::mirrored` -- and the bake wants a recipe per clip.
    //
    // The two Lances are one input -- middle click -- and the force she is
    // carrying decides which comes out. **They are two clips because the whole
    // point is that they do not look alike**: a person standing opposite has
    // to be able to tell a burst they should get out from under from a tether
    // they should break, and the only place that can be read is the wind-up.
    DualDark,      "dual_dark",       Moves, "dual", Length::Move(Class::DualMage, 0), false,
        "Dark auto, left arm: a straight punch that opens into a wing behind the fist.";
    DualLightLance,"dual_light_lance",Moves, "dual", Length::Move(Class::DualMage, 1), false,
        "Light lance: the right arm cocked high and thrown through, a long line aimed past them to burst behind.";
    DualSpecial,   "dual_special",    Moves, "dual", Length::Move(Class::DualMage, 2), false,
        "Judgement: the finisher. Squared up, both forces at once, and as big as the bar lets it be.";
    DualSweep,     "dual_sweep",      Moves, "dual", Length::Move(Class::DualMage, 3), false,
        "Sweep: both arms thrown across the whole front at once, round past both shoulders. Close, wide, and it moves people.";
    DualLight,     "dual_light",      Moves, "dual", Length::Move(Class::DualMage, 4), false,
        "Light auto, right arm: the dark punch mirrored, and it has to read as the other side.";
    DualDarkLance, "dual_dark_lance", Moves, "dual", Length::Move(Class::DualMage, 5), false,
        "Dark lance: the left arm hauled low and back, a slow reach that ends holding on rather than letting go.";
}

impl Clip {
    /// How many frames this clip is. Attack clips follow their move's live
    /// frame data; everything else is the count declared above.
    pub fn length(self) -> u16 {
        match self.length_spec() {
            Length::Fixed(n) => n,
            Length::Move(class, slot) => {
                let (s, a, r) = sim::moves::frames(class, slot);
                (s + a + r).max(2)
            }
            Length::Dodge => sim::tuning::dodge_frames().max(2),
            Length::AirDodge => sim::tuning::air_dodge_frames().max(2),
            Length::ShadowFlight => sim::tuning::shadow_send_frames().max(2),
        }
    }

    /// The move this clip animates, if it animates one.
    pub fn move_slot(self) -> Option<(Class, u8)> {
        match self.length_spec() {
            Length::Move(c, s) => Some((c, s)),
            _ => None,
        }
    }

    /// The three phase boundaries of an attack clip, in frames: the last
    /// startup frame, the first active frame, and the first recovery frame.
    ///
    /// Authoring against these rather than against typed-in frame numbers is
    /// what keeps the contact pose on the frame the hitbox actually appears.
    pub fn phases(self) -> Option<(u16, u16, u16)> {
        let (class, slot) = self.move_slot()?;
        let (s, a, _) = sim::moves::frames(class, slot);
        Some((s.saturating_sub(1), s, s + a))
    }

    pub fn index(self) -> usize {
        ALL.iter().position(|c| *c == self).unwrap_or(0)
    }

    /// Look a clip up by the name its recipe uses.
    pub fn by_name(name: &str) -> Option<Clip> {
        ALL.iter().copied().find(|c| c.name() == name)
    }

    /// The frames, from the baked table.
    pub fn frames(self) -> &'static [crate::pose::Pose] {
        crate::baked::TABLE[self.index()]
    }

    /// The pose at a frame, clamped or wrapped depending on whether the clip
    /// loops. Playback is a lookup, which is what keeps it a pure function of
    /// simulation state.
    pub fn at(self, frame: u32) -> crate::pose::Pose {
        let frames = self.frames();
        if frames.is_empty() {
            return crate::pose::Pose::rest();
        }
        let i = if self.looping() {
            (frame as usize) % frames.len()
        } else {
            (frame as usize).min(frames.len() - 1)
        };
        frames[i]
    }

    /// Sample at a fractional frame, blending between neighbours. Locomotion
    /// clips are played at a rate that depends on speed, so they land between
    /// frames constantly and a nearest-frame lookup visibly stutters.
    pub fn at_fractional(self, frame: f32) -> crate::pose::Pose {
        let frames = self.frames();
        if frames.is_empty() {
            return crate::pose::Pose::rest();
        }
        let n = frames.len();
        if self.looping() {
            let f = frame.rem_euclid(n as f32);
            let i = f.floor() as usize % n;
            let k = f - f.floor();
            frames[i].blend(&frames[(i + 1) % n], k)
        } else {
            let f = frame.clamp(0.0, (n - 1) as f32);
            let i = f.floor() as usize;
            let k = f - f.floor();
            if i + 1 >= n {
                frames[n - 1]
            } else {
                frames[i].blend(&frames[i + 1], k)
            }
        }
    }
}

/// Every clip a given class can be asked to play: the shared vocabulary plus
/// its own three moves. Used by the hub's list and by a test that checks the
/// class has no gaps.
pub fn for_class(class: Class) -> Vec<Clip> {
    ALL.iter()
        .copied()
        .filter(|c| match c.move_slot() {
            Some((owner, _)) => owner == class,
            None => true,
        })
        .collect()
}
