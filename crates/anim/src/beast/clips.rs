//! Every animation the Ridgeback has, as poses and timing.
//!
//! The shape of a move is a promise to the player about what is coming, so
//! these are written against the move table rather than against frame numbers:
//! [`mark`] turns "the start of the active window" into wherever that currently
//! falls, and the bake resamples into phases. Retune a move in the Oven and its
//! animation stretches with it.
//!
//! **The house style is Monster Hunter's**, and it is three rules:
//!
//! 1. **The windup is the move.** Most of the frames are before the hit, the
//!    silhouette changes early and hugely, and the pose at the top of it says
//!    exactly what is about to happen and roughly where. A player who has seen
//!    a move twice should be able to name it from frame five.
//! 2. **The hit is short and enormous.** A few frames, travelling further than
//!    looks survivable, with the whole body behind it.
//! 3. **The recovery is the fight.** It is long, it is where you are allowed to
//!    be, and what the animal is doing during it is the answer to the question
//!    of where to stand. The sweep's tail lies on the floor through its
//!    recovery, and that is a staircase.

use super::{Key, Looseness, Pose, Recipe, mark};
use crate::ease::Ease;
use sim::beast::Clip;

pub fn all() -> Vec<Recipe> {
    vec![
        idle(),
        walk(),
        gallop(),
        bite(),
        stomp(),
        sweep(),
        charge(),
        slam(),
        shake(),
        kick(),
        flinch(),
        stumble(),
        topple(),
        dead(),
    ]
}

// ---------------------------------------------------------------------------
// Standing and moving
// ---------------------------------------------------------------------------

fn idle() -> Recipe {
    // Breathing, and nothing else. It is the pose the player spends the most
    // time looking at, so the only thing it must not do is hold still: a
    // creature frozen between moves reads as a prop, and the whole fight is
    // about reading it as alive enough to have intentions.
    let breathe = |k: f32| {
        Pose::standing()
            .hips(0.0, 0.05 * k, 0.0)
            .spine(1.2 * k, 0.0, 0.0)
            .chest(1.6 * k, 0.0, 0.0)
            .neck(2.5 * k, 1.5 * k)
            .tail(3.0 * k, 5.0 * k)
    };
    Recipe::new(
        Clip::Idle,
        vec![
            Key::at(0.0, breathe(-1.0)),
            Key::at(0.5, breathe(1.0)),
            Key::at(0.75, breathe(0.3)),
        ],
        Looseness::BEAST,
    )
    .noting(
        "Breathing. Two beats a cycle with the tail drifting on its own, \
         because the tail's own lag is what stops the loop reading as a \
         metronome. Nothing here is gameplay -- but a creature that holds \
         perfectly still between moves reads as a prop, and the fight needs it \
         to read as an animal deciding something."
            .to_string(),
    )
}

/// One leg at a point in its own cycle.
///
/// `u` is the leg's local phase, zero at touchdown. Stance is most of it: the
/// foot is on the ground and the body travels over it, which from the hip looks
/// like the leg sweeping backward. The rest is swing, and the knee folds to get
/// the foot off the floor.
///
/// Arithmetic rather than keys, deliberately. Four legs times two joints times
/// eight keys is sixty-four numbers nobody can hold in their head, and the
/// thing that makes a gait read is the *relationship* between them, which a
/// function states and a table of numbers only implies.
fn stride(p: Pose, which: usize, u: f32, reach: f32, lift: f32) -> Pose {
    const STANCE: f32 = 0.62;
    let u = u.rem_euclid(1.0);
    if u < STANCE {
        // **Planted.** The foot is on the floor and the body travels over it,
        // which from the hip looks like the leg sweeping backward. Solved for a
        // foot position rather than keyed as angles, so it lands on the floor
        // rather than near it -- see `Pose::plant`.
        let k = u / STANCE;
        p.plant(which, reach * (1.0 - 2.0 * k), 0.0)
    } else {
        // Swinging: forward again, lifted most at mid-swing.
        let k = (u - STANCE) / (1.0 - STANCE);
        let arc = (k * std::f32::consts::PI).sin();
        p.plant(which, reach * (2.0 * k - 1.0), lift * arc)
    }
}

fn walk() -> Recipe {
    // A four-beat lateral sequence -- hind, then fore, on the same side, then
    // the other side. It is the gait that makes a heavy quadruped read as
    // heavy: three feet are on the ground at any moment, so the body never
    // leaves it, and the weight visibly transfers rather than floating.
    const OFFSET: [f32; 4] = [0.5, 0.0, 0.75, 0.25];
    let at = |t: f32| {
        let mut p = Pose::standing();
        for (which, offset) in OFFSET.into_iter().enumerate() {
            p = stride(p, which, t + offset, 1.1, 0.55);
        }
        // Two shallow dips a cycle, as each shoulder takes the load, and a
        // lateral sway a quarter cycle out of step with them. The sway is what
        // makes it walk rather than trot in place.
        let bob = -0.055 * (2.0 * (t * std::f32::consts::TAU).sin().abs());
        let sway = 3.2 * ((t + 0.25) * std::f32::consts::TAU).sin();
        p.hips(0.0, bob, 0.0)
            .root(1.0 * (t * std::f32::consts::TAU).cos(), 0.0, sway * 0.4)
            .spine(0.0, sway * 0.5, 0.0)
            .chest(0.0, -sway * 0.4, 0.0)
            .neck(-1.5, -sway * 0.8)
            .tail(2.0, sway * 2.6)
    };
    Recipe::new(
        Clip::Walk,
        (0..8)
            .map(|i| {
                let t = i as f32 / 8.0;
                Key::eased(t, at(t), Ease::LINEAR)
            })
            .collect(),
        Looseness::GAIT,
    )
    .noting(
        "The walk, as a four-beat lateral sequence: hind then fore on one side, \
         then the other. Three feet down at all times, which is what makes it \
         read as heavy. Linear between keys on purpose -- a stride is the one \
         motion in the game that genuinely is constant speed, and easing it is \
         what makes a walk look like it is being carried."
            .to_string(),
    )
}

fn gallop() -> Recipe {
    // Not a faster walk. A gallop is a gather and a throw: the hind pair drive
    // together, the forelegs catch, and the spine flexes and extends through
    // both -- which is where the reach comes from and why a galloping animal
    // covers ground a walking one cannot.
    const OFFSET: [f32; 4] = [0.42, 0.5, 0.0, 0.08];
    let at = |t: f32| {
        let mut p = Pose::standing();
        for (which, offset) in OFFSET.into_iter().enumerate() {
            p = stride(p, which, t + offset, 1.9, 1.15);
        }
        let wheel = t * std::f32::consts::TAU;
        // One big rise and fall, and a spine that folds under at the gather and
        // opens out at full extension.
        let bob = 0.30 * (wheel - 0.9).sin() - 0.05;
        let flex = 9.0 * (wheel - 0.4).sin();
        p.hips(0.0, bob, 0.0)
            .root(-flex * 0.5, 0.0, 0.0)
            .spine(flex, 0.0, 0.0)
            .chest(flex * 0.7, 0.0, 0.0)
            .neck(-10.0 - flex * 0.8, 0.0)
            .tail(14.0 - flex, 0.0)
    };
    Recipe::new(
        Clip::Gallop,
        (0..8)
            .map(|i| {
                let t = i as f32 / 8.0;
                Key::eased(t, at(t), Ease::LINEAR)
            })
            .collect(),
        Looseness::GAIT,
    )
    .noting(
        "The gallop. Hind pair together, forelegs catching, and a spine that \
         folds at the gather and opens at extension. The suspension is real -- \
         the hips rise a third of a metre -- which is what makes a charge feel \
         like something arriving rather than something sliding."
            .to_string(),
    )
}

// ---------------------------------------------------------------------------
// The moves
// ---------------------------------------------------------------------------

fn bite() -> Recipe {
    let ready = Pose::standing();
    // The coil. Head drawn back and high, weight onto the hind legs, neck
    // folded like a spring -- from across the arena this is a completely
    // different silhouette from anything else it does, which is the point.
    let coil = Pose::standing()
        .hips(-0.14, 0.05, 0.0)
        .root(6.0, 0.0, 0.0)
        .spine(5.0, 0.0, 0.0)
        .chest(5.0, 0.0, 0.0)
        .neck(32.0, 0.0)
        .head(-22.0, 0.0, 0.0)
        .tail_lift(16.0)
        .plant_fore(-0.12, 0.0)
        .plant_hind(-0.50, 0.0);
    // Thrown. The neck unfolds completely and the head goes past where it looks
    // like it can, which is most of what makes a lunge read as committed.
    let strike = Pose::standing()
        .hips(0.16, -0.05, 0.0)
        .root(-5.0, 0.0, 0.0)
        .spine(-7.0, 0.0, 0.0)
        .chest(-6.0, 0.0, 0.0)
        .neck(-28.0, 0.0)
        .head(14.0, 0.0, 0.0)
        .tail_lift(-10.0)
        .plant_fore(0.34, 0.0)
        .plant_hind(-0.10, 0.0);
    Recipe::new(
        Clip::Bite,
        vec![
            Key::eased(0.0, ready, Ease::ANTICIPATE),
            Key::eased(mark(Clip::Bite, 0, 0.55), coil, Ease::SNAP),
            Key::eased(mark(Clip::Bite, 1, 0.0), strike, Ease::STRIKE),
            Key::eased(mark(Clip::Bite, 1, 1.0), strike, Ease::OUT),
            Key::eased(mark(Clip::Bite, 2, 0.45), ready, Ease::SMOOTH),
            Key::at(1.0, ready),
        ],
        Looseness::SNAP,
    )
    .noting(
        "Coil and throw. The head is drawn back and *up* rather than straight \
         back, so the shape is legible from the side as well as from in front, \
         and it holds there -- SNAP out of the coil -- for long enough to be \
         read. The strike arrives on the first active frame and everything \
         after it is the neck ringing down."
            .to_string(),
    )
}

fn stomp() -> Recipe {
    let ready = Pose::standing();
    // The fastest thing it does, so the tell has to be *big* rather than long:
    // one foreleg comes up past the shoulder while the weight rocks back.
    let raised = Pose::standing()
        .hips(-0.06, 0.04, 0.0)
        .root(5.0, 0.0, -2.0)
        .chest(3.0, 0.0, 0.0)
        .neck(9.0, 0.0)
        .plant(0, -0.10, 0.0)
        .plant_hind(-0.42, 0.0)
        .leg(1, 66.0, -74.0);
    let driven = Pose::standing()
        .hips(0.04, -0.09, 0.0)
        .root(-5.0, 0.0, 1.5)
        .chest(-4.0, 0.0, 0.0)
        .neck(-12.0, 0.0)
        .tail_lift(12.0)
        .plant(0, -0.06, 0.0)
        .plant(1, 0.62, 0.0)
        .plant_hind(-0.24, 0.0);
    Recipe::new(
        Clip::Stomp,
        vec![
            Key::eased(0.0, ready, Ease::IN),
            Key::eased(mark(Clip::Stomp, 0, 0.75), raised, Ease::SNAP),
            Key::eased(mark(Clip::Stomp, 1, 0.0), driven, Ease::STRIKE),
            Key::eased(mark(Clip::Stomp, 2, 0.35), driven, Ease::OUT),
            Key::at(1.0, ready),
        ],
        Looseness::SNAP,
    )
    .noting(
        "Fifteen frames of startup is under reaction, so this one is answered \
         by *position* rather than by sight -- do not stand in front of it. \
         The tell is therefore as large as it can be made in the frames \
         available: the leg goes above the shoulder line, which is a change to \
         the silhouette rather than a change to a pose."
            .to_string(),
    )
}

fn sweep() -> Recipe {
    let ready = Pose::standing();
    // Wound. The tail goes one way and the shoulders go the other -- a tail
    // that heavy cannot be swung without the rest of the animal paying for it,
    // and the counter-rotation is most of what makes it read as weight.
    let wound = Pose::standing()
        .hips(0.0, -0.08, 0.0)
        .root(2.0, 9.0, -6.0)
        .spine(0.0, 7.0, 0.0)
        .chest(0.0, 6.0, 0.0)
        .neck(6.0, 14.0)
        .tail(18.0, -40.0)
        .tail_swing_base(-34.0)
        .tail_roll(-26.0)
        .plant_fore(0.02, 0.0)
        .plant_hind(-0.38, 0.0);
    // Through. The tail is low and past the far side, and the tip is carrying
    // all of the speed -- which is why the hitbox is ankle height and why
    // jumping it is the answer.
    let through = Pose::standing()
        .hips(0.0, -0.16, 0.0)
        .root(-3.0, -11.0, 7.0)
        .spine(0.0, -9.0, 0.0)
        .chest(0.0, -7.0, 0.0)
        .neck(-4.0, -16.0)
        .tail(-16.0, 46.0)
        .tail_swing_base(40.0)
        .tail_roll(30.0)
        .plant_fore(0.20, 0.0)
        .plant_hind(-0.18, 0.0);
    // And down. The tail lies on the floor through the recovery, which is the
    // one route onto its back that needs neither a platform nor a broken leg.
    let dropped = Pose::standing()
        .hips(0.0, -0.10, 0.0)
        .root(-2.0, -4.0, 2.0)
        .neck(-2.0, -6.0)
        .tail(-34.0, 16.0)
        .tail_swing_base(14.0)
        .tail_roll(8.0)
        .plant_fore(0.10, 0.0)
        .plant_hind(-0.28, 0.0);
    Recipe::new(
        Clip::Sweep,
        vec![
            Key::eased(0.0, ready, Ease::ANTICIPATE),
            Key::eased(mark(Clip::Sweep, 0, 0.6), wound, Ease::SNAP),
            Key::eased(mark(Clip::Sweep, 1, 0.25), through, Ease::OUT),
            Key::eased(mark(Clip::Sweep, 2, 0.2), dropped, Ease::HOLD),
            Key::eased(mark(Clip::Sweep, 2, 0.7), dropped, Ease::SMOOTH),
            Key::at(1.0, ready),
        ],
        Looseness::HEAVY,
    )
    .noting(
        "Wind, whip, and leave it there. The recovery is the important half: \
         the tail is on the floor for most of it and the animal is slow to pick \
         it up, so the sweep you just jumped is also a staircase. Held with \
         HOLD rather than eased out of, because a tail that starts lifting \
         immediately is a staircase that is not there by the time you reach it."
            .to_string(),
    )
}

fn charge() -> Recipe {
    let ready = Pose::standing();
    // Gathered. Haunches loaded, head down and level, forelegs braced -- a
    // sprinter in the blocks, which is a shape everybody reads instantly.
    let gathered = Pose::standing()
        .hips(-0.45, -0.18, 0.0)
        .root(9.0, 0.0, 0.0)
        .spine(6.0, 0.0, 0.0)
        .chest(-8.0, 0.0, 0.0)
        .neck(-22.0, 0.0)
        .head(10.0, 0.0, 0.0)
        .tail_lift(20.0)
        .plant_fore(-0.30, 0.0)
        .plant_hind(-0.62, 0.0);
    let running = Pose::standing()
        .hips(0.0, 0.06, 0.0)
        .root(-4.0, 0.0, 0.0)
        .chest(-10.0, 0.0, 0.0)
        .neck(-16.0, 0.0)
        .head(12.0, 0.0, 0.0)
        .tail_lift(16.0)
        .plant_fore(0.85, 0.30)
        .plant_hind(-0.70, 0.0);
    // The skid. Weight back, forelegs out in front, head up: the recovery is
    // long and it is completely unmistakeable, which is the whole reason a
    // charge is worth baiting.
    let skid = Pose::standing()
        .hips(-0.30, -0.20, 0.0)
        .root(11.0, 0.0, 0.0)
        .spine(4.0, 0.0, 0.0)
        .chest(6.0, 0.0, 0.0)
        .neck(20.0, 0.0)
        .tail_lift(-18.0)
        .plant_fore(0.95, 0.0)
        .plant_hind(-0.55, 0.0);
    Recipe::new(
        Clip::Charge,
        vec![
            Key::eased(0.0, ready, Ease::IN),
            Key::eased(mark(Clip::Charge, 0, 0.7), gathered, Ease::SNAP),
            Key::eased(mark(Clip::Charge, 1, 0.08), running, Ease::LINEAR),
            Key::eased(mark(Clip::Charge, 1, 1.0), running, Ease::OUT),
            Key::eased(mark(Clip::Charge, 2, 0.25), skid, Ease::SMOOTH),
            Key::at(1.0, ready),
        ],
        Looseness::HEAVY,
    )
    .noting(
        "Load, run, and skid. The gallop cycle is layered over the run by the \
         simulation rather than keyed here -- the creature is genuinely moving \
         at fifteen metres a second during the active window, so its legs \
         should come from the ground it is covering. What this clip supplies is \
         the carriage: head down and level through the run, and the long, \
         obvious skid that is the reason to bait one."
            .to_string(),
    )
}

fn slam() -> Recipe {
    let ready = Pose::standing();
    // Rearing. Fifty-two frames of startup is nearly a second, and it is all
    // spent going up -- the animal gets bigger, which is the most legible thing
    // a silhouette can do.
    let half = Pose::standing()
        .hips(-0.35, 0.16, 0.0)
        .root(22.0, 0.0, 0.0)
        .spine(8.0, 0.0, 0.0)
        .chest(6.0, 0.0, 0.0)
        .neck(16.0, 0.0)
        .tail_lift(26.0)
        .plant_hind(-0.30, 0.0)
        .forelegs(-34.0, -46.0);
    let reared = Pose::standing()
        .hips(-0.55, 0.34, 0.0)
        .root(44.0, 0.0, 0.0)
        .spine(12.0, 0.0, 0.0)
        .chest(8.0, 0.0, 0.0)
        .neck(24.0, 0.0)
        .head(-16.0, 0.0, 0.0)
        .tail_lift(50.0)
        .plant_hind(-0.24, 0.0)
        .forelegs(-58.0, -78.0);
    // Down. Everything reverses at once, which is why this is the move you
    // leave the back for rather than the one you brace against.
    // The crash goes a metre further than the legs' own length asks, so the
    // shoulders at the bottom of it stay inside a standing jump: the answer
    // to the hardest-hitting move in the set is still an invitation.
    let slammed = Pose::standing()
        .hips(0.40, -1.05, 0.0)
        .root(-13.0, 0.0, 0.0)
        .spine(-10.0, 0.0, 0.0)
        .chest(-14.0, 0.0, 0.0)
        .neck(-14.0, 0.0)
        .tail_lift(24.0)
        .plant_fore(0.72, 0.0)
        .plant_hind(-0.20, 0.0);
    Recipe::new(
        Clip::Slam,
        vec![
            Key::eased(0.0, ready, Ease::ANTICIPATE),
            Key::eased(mark(Clip::Slam, 0, 0.45), half, Ease::OUT),
            Key::eased(mark(Clip::Slam, 0, 0.8), reared, Ease::HOLD),
            Key::eased(mark(Clip::Slam, 1, 0.0), slammed, Ease::STRIKE),
            Key::eased(mark(Clip::Slam, 2, 0.3), slammed, Ease::SMOOTH),
            Key::at(1.0, ready),
        ],
        Looseness::HEAVY,
    )
    .noting(
        "The long telegraph, and the one clip where the shape of the *time* \
         matters more than the poses. It rises over the first half, reaches the \
         top early, and then HOLDs there -- a creature standing on its hind \
         legs and not moving is the single loudest thing in the fight, and it \
         is loud for a fifth of a second before anything happens. Everything \
         reverses on one frame after that, which is what no brace survives."
            .to_string(),
    )
}

fn shake() -> Recipe {
    // **The bait.** Forty frames of startup that are emphatically not violent:
    // it plants its feet, hunches, and leans. A rider has to be able to tell
    // this apart from the shake itself, because the whole strategy is to leave
    // the ground over the second half and not the first.
    let plant = Pose::standing()
        .hips(0.0, -0.18, 0.0)
        .root(4.0, 0.0, 0.0)
        .spine(-4.0, 0.0, 0.0)
        .chest(-6.0, 0.0, 0.0)
        .neck(-10.0, 0.0)
        .tail_lift(10.0)
        .plant_fore(-0.06, 0.0)
        .plant_hind(-0.46, 0.0);
    let lean = plant.root(4.0, 7.0, -12.0).spine(-4.0, 5.0, 0.0);
    // The shake itself: the spine whipping about its own length. Roll rather
    // than yaw does most of the work, because rolling is what throws somebody
    // standing *on top* and yawing mostly slides them.
    // **The hips hold the value the plant left them at.** A step in the pose
    // between two frames is an arbitrarily large acceleration, and an
    // arbitrarily large acceleration throws everybody regardless of what the
    // move was supposed to do -- the haunch is welded to the hips, so an eight
    // centimetre jump here made the one patch of animal directly over the pivot
    // the most violent place on it, which is backwards.
    let whip = |k: f32| {
        Pose::standing()
            .hips(0.0, -0.18, 0.0)
            // **The hips stay planted and the back does the shaking.** The
            // haunch is welded to the root, so putting the whip there made the
            // one patch of animal directly over the pivot the most violent
            // place to stand -- a brace would not hold on the flattest, widest,
            // most obvious spot on it. Driving the spine and chest instead
            // gives the gradient the shape it should have had all along: calm
            // at the hips, worse the further along the back you are.
            .root(2.0, 3.0 * k, -3.0 * k)
            .spine(0.0, 11.0 * k, -50.0 * k)
            .chest(0.0, 8.0 * k, -26.0 * k)
            .neck(-4.0, -14.0 * k)
            .tail_swing_base(-18.0 * k)
            .tail(4.0, -14.0 * k)
            .plant_fore(-0.02, 0.0)
            .plant_hind(-0.40, 0.0)
    };
    let mut keys = vec![
        Key::eased(0.0, Pose::standing(), Ease::IN),
        Key::eased(mark(Clip::Shake, 0, 0.55), plant, Ease::SMOOTH),
        // Not at 1.0: the first whip key sits at the start of the active
        // window, and two keys at the same instant with different poses are a
        // *step* rather than a motion -- an arbitrarily large acceleration on
        // one frame, which threw a braced rider off the hips before the shake
        // had started. The lean lands just short and the whip is the thing
        // that travels.
        Key::eased(mark(Clip::Shake, 0, 0.88), lean, Ease::SNAP),
    ];
    // Nine reversals across the active window. Each is a full swing from one
    // side to the other, so the *acceleration* -- which is what the grip test
    // actually measures -- peaks at every crossing.
    //
    // The count is tied to the window's length rather than chosen for looks:
    // it has to stay violent per reversal however long the shake is tuned to
    // run, and spreading a fixed number of swings over a longer window makes
    // each of them slower and the whole move gentler.
    // Six for a thirty-six frame whip: six frames a swing, which is about
    // what nine were over the fifty-six frame whip it had until 2026-09-23.
    // The whip was shortened so that a hop begun late in the tell lands
    // after it -- the jump-the-shake read the ride is built around, which a
    // whip longer than a hop had quietly made impossible -- and nine swings
    // in thirty-six frames threw braced riders off the hips, which are
    // supposed to be the calm place.
    const REVERSALS: usize = 6;
    for i in 0..=REVERSALS {
        let u = i as f32 / REVERSALS as f32;
        let side = if i % 2 == 0 { 1.0 } else { -1.0 };
        keys.push(Key::eased(
            mark(Clip::Shake, 1, u),
            whip(side),
            Ease::SMOOTH,
        ));
    }
    keys.push(Key::eased(
        mark(Clip::Shake, 2, 0.5),
        Pose::standing().hips(0.0, -0.12, 0.0).tail_lift(6.0),
        Ease::OUT,
    ));
    keys.push(Key::at(1.0, Pose::standing()));
    Recipe::new(Clip::Shake, keys, Looseness::BUCK).noting(
        "The one move whose animation is a *contract*. Forty frames of planting \
         and leaning, which are deliberately not violent, and then six \
         reversals that throw anybody still standing on it. A rider is meant to \
         be able to read the first and jump the second, and the only reason \
         that is possible is that the startup does not shake. An early version \
         oscillated through the windup and threw riders before the telegraph \
         had finished being a telegraph, which is the one thing a long startup \
         is for. \
         Roll rather than yaw carries it, because rolling is what throws \
         somebody standing on top of you and yawing mostly slides them."
            .to_string(),
    )
}

fn kick() -> Recipe {
    let ready = Pose::standing();
    // Weight onto the forelegs, head down, hind feet gathered under the belly.
    // The tail comes up out of the way -- which is the tell that reads from
    // behind, where the person this move is for is standing: a tail lifting
    // and a rump dropping are the two things in the silhouette they can see.
    let gather = Pose::standing()
        .hips(0.30, -0.40, 0.0)
        .root(-9.0, 0.0, 0.0)
        .spine(-4.0, 0.0, 0.0)
        .chest(-3.0, 0.0, 0.0)
        .neck(-12.0, 0.0)
        .head(6.0, 0.0, 0.0)
        .tail_lift(34.0)
        .plant_fore(0.04, 0.0)
        .plant_hind(0.30, 0.0);
    // Both hind legs straight back and up. The feet leave the floor for the
    // only time in the move set, which is why it is fast: there is no stance
    // to change, the animal simply throws the back half of itself.
    let thrown = Pose::standing()
        .hips(0.20, -0.30, 0.0)
        .root(-14.0, 0.0, 0.0)
        .spine(-3.0, 0.0, 0.0)
        .chest(-2.0, 0.0, 0.0)
        .neck(-14.0, 0.0)
        .head(8.0, 0.0, 0.0)
        .tail_lift(46.0)
        .plant_fore(0.0, 0.0)
        .hindlegs(-68.0, 8.0);
    // Down again, heavily. The hind end drops onto legs that were not under
    // it, and the recovery is the animal getting its feet back.
    let landed = Pose::standing()
        .hips(0.06, -0.28, 0.0)
        .root(-5.0, 0.0, 0.0)
        .neck(-6.0, 0.0)
        .tail_lift(14.0)
        .plant_fore(0.10, 0.0)
        .plant_hind(-0.36, 0.0);
    Recipe::new(
        Clip::Kick,
        vec![
            Key::eased(0.0, ready, Ease::ANTICIPATE),
            Key::eased(mark(Clip::Kick, 0, 0.7), gather, Ease::SNAP),
            Key::eased(mark(Clip::Kick, 1, 0.0), thrown, Ease::STRIKE),
            Key::eased(mark(Clip::Kick, 1, 1.0), thrown, Ease::OUT),
            Key::eased(mark(Clip::Kick, 2, 0.4), landed, Ease::SMOOTH),
            Key::at(1.0, ready),
        ],
        Looseness::SNAP,
    )
    .noting(
        "The mule kick, and it exists for one patch of floor: directly behind \
         the hips, where every forward move needs you elsewhere and the sweep \
         is a whip about the very point you are standing on. Twenty-two frames \
         of startup is readable but only just, and the tell is authored for \
         the person it is aimed at -- who is behind it and cannot see the head \
         -- so it is the tail going up and the rump going down. Its answer is \
         a sidestep, which is not the sweep's answer, so being behind the \
         animal is now two reads rather than none."
            .to_string(),
    )
}

// ---------------------------------------------------------------------------
// Being in trouble
// ---------------------------------------------------------------------------

fn flinch() -> Recipe {
    let hit = Pose::standing()
        .hips(-0.12, -0.06, 0.0)
        .root(5.0, -4.0, 4.0)
        .spine(-6.0, -3.0, 0.0)
        .neck(-16.0, -8.0)
        .head(9.0, 0.0, 0.0)
        .tail(10.0, -14.0)
        .plant_fore(-0.04, 0.0)
        .plant_hind(-0.40, 0.0);
    Recipe::new(
        Clip::Flinch,
        vec![
            Key::eased(0.0, hit, Ease::OUT),
            Key::eased(0.45, hit.blend(&Pose::standing(), 0.5), Ease::SMOOTH),
            Key::at(1.0, Pose::standing()),
        ],
        Looseness::LIMP,
    )
    .noting(
        "A recoil, held at the start and bled off. Indexed from the beginning \
         rather than the end, unlike a fighter's hitstun, because what matters \
         here is that the animal is visibly *hit* on the frame it is hit -- the \
         creature's own frame counter is what says when it can act again."
            .to_string(),
    )
}

fn stumble() -> Recipe {
    // Down on a knee. Authored as a general collapse -- the front end is what
    // usually goes, and the simulation adds the pitch that says which end -- so
    // that a broken hindfoot and a broken forefoot share one clip and still
    // read differently.
    // **A metre deeper than the legs used to need**, since 2026-09-23: the
    // legs grew a metre and the shoulder still has to come down to about two
    // and a half, because that number is what a broken foot buys and it is
    // measured against the roster's jumps, not against the animal's height.
    let buckling = Pose::standing()
        .hips(0.10, -0.95, 0.0)
        .root(-6.0, 0.0, 5.0)
        .spine(-7.0, 0.0, 0.0)
        .chest(-10.0, 0.0, 0.0)
        .neck(-12.0, 8.0)
        .tail(14.0, -18.0)
        // Planted a little *above* the floor, because a metre and a half of
        // drop through LIMP springs overshoots and a foot solved onto the
        // floor at the key goes through it between keys.
        .plant_hind(-0.42, 0.45)
        .plant(1, 0.42, 0.45)
        // The kneeling leg is keyed as angles because it is the one leg that
        // is *not* on the floor; the shin folds back to nearly horizontal so
        // that a lower leg over two metres long does not go through it.
        .leg(0, 30.0, -112.0);
    let down = Pose::standing()
        .hips(0.05, -1.62, 0.0)
        .root(-13.0, 0.0, 9.0)
        .spine(-10.0, 0.0, 0.0)
        .chest(-16.0, 0.0, 0.0)
        .neck(-12.0, 12.0)
        .head(8.0, 0.0, 0.0)
        .tail(6.0, -10.0)
        .plant_hind(-0.50, 0.55)
        .plant(1, 0.55, 0.55)
        .leg(0, 42.0, -134.0);
    Recipe::new(
        Clip::Stumble,
        vec![
            Key::eased(0.0, Pose::standing(), Ease::IN),
            Key::eased(0.12, buckling, Ease::OUT),
            Key::eased(0.26, down, Ease::HOLD),
            Key::eased(0.66, down, Ease::SMOOTH),
            Key::eased(0.88, buckling, Ease::OUT),
            Key::at(1.0, Pose::standing()),
        ],
        Looseness::COLLAPSE,
    )
    .noting(
        "The mount window, and it is shaped to be one: it goes down over a \
         quarter of the clip, HOLDs at the bottom for most of it, and is slow \
         getting back up. A collapse that bounced straight back would be a \
         window you cannot cross the arena to reach, which is the same as no \
         window at all. \
         The shoulder comes down to about two and a half metres, which is a \
         standing jump for most of the roster -- `beastcheck` prints it."
            .to_string(),
    )
}

fn topple() -> Recipe {
    // Onto its side. The only pose in the set with real roll in it, and that is
    // why it reads as a different *kind* of event from everything else: nothing
    // the creature chooses to do rolls it.
    let over = Pose::standing()
        .hips(0.0, -2.55, 0.35)
        .root(-6.0, 0.0, 62.0)
        .spine(-4.0, 0.0, 10.0)
        .chest(-6.0, 0.0, 8.0)
        .neck(-26.0, 22.0)
        .head(20.0, 0.0, 0.0)
        .forelegs(28.0, -40.0)
        .hindlegs(40.0, -56.0)
        .leg_spread(0, 26.0)
        .leg_spread(1, 26.0)
        .leg_spread(2, 22.0)
        .leg_spread(3, 22.0)
        .tail(4.0, 34.0);
    let heaving = over.blend(&Pose::standing(), 0.45).hips(0.0, -1.4, 0.15);
    Recipe::new(
        Clip::Topple,
        vec![
            Key::eased(0.0, Pose::standing(), Ease::IN),
            Key::eased(0.14, over, Ease::HOLD),
            Key::eased(0.62, over, Ease::SMOOTH),
            Key::eased(0.82, heaving, Ease::OUT),
            Key::at(1.0, Pose::standing()),
        ],
        Looseness::COLLAPSE,
    )
    .noting(
        "A fall is fast and standing up is not, and the key positions say so: \
         over in a seventh of the clip, on the floor for half of it, and the \
         last fifth spent heaving back up. This is the window the whole climb \
         exists to earn, so it is the one clip where the *length* of the hold \
         is the design rather than the look of the pose."
            .to_string(),
    )
}

fn dead() -> Recipe {
    let gone = Pose::standing()
        .hips(0.0, -2.70, 0.45)
        .root(-8.0, 0.0, 74.0)
        .spine(-6.0, 0.0, 12.0)
        .chest(-8.0, 0.0, 10.0)
        .neck(-34.0, 30.0)
        .head(26.0, 0.0, 0.0)
        .forelegs(34.0, -34.0)
        .hindlegs(46.0, -50.0)
        .leg_spread(0, 30.0)
        .leg_spread(1, 30.0)
        .leg_spread(2, 26.0)
        .leg_spread(3, 26.0)
        .tail(2.0, 42.0);
    Recipe::new(Clip::Dead, vec![Key::at(0.0, gone)], Looseness::LIMP).noting(
        "One pose. There is nothing after this that anybody is watching, and a \
         death animation the hunt does not wait for is an animation nobody sees \
         the end of."
            .to_string(),
    )
}

/// Every clip that has no recipe. Empty is the answer we want, and the bake
/// prints this so it cannot go quietly stale.
pub fn missing() -> Vec<Clip> {
    let have = all();
    Clip::ALL
        .into_iter()
        .filter(|c| !have.iter().any(|r| r.clip == *c))
        .collect()
}
