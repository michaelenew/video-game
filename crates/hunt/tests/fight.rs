//! Is the fight any good?
//!
//! The same trick as `crates/sim/tests/feel.rs`: "feels good" cannot be a test,
//! but the properties a good fight needs can be. These pin **relationships**,
//! not values -- every number in the Oven should move freely, and a failure
//! here means either a bug or a design decision that changed without
//! `docs/design/monsters.md` changing with it.
//!
//! Everything is measured over several seeds. A creature that chooses produces
//! a distribution, and one run of a distribution is an anecdote.

use hunt::{Outcome, Report, play, report::REACTION_NOTE, report::Threat};
use sim::monster::{self, MOVES};
use sim::moves;
use sim::state::MAX_PLAYERS;

/// Long enough to reach a conclusion, short enough that a grind shows up as a
/// failure rather than as a slow test.
const BUDGET: u32 = 36_000;

const SEEDS: [u32; 6] = [1, 7, 101, 2_222, 60_013, 0x2545_F491];

fn hunts() -> Vec<Report> {
    SEEDS
        .iter()
        .map(|s| play([sim::Class::Champion; MAX_PLAYERS], 1, BUDGET, *s))
        .collect()
}

// ---------------------------------------------------------------------------
// Does the fight have decisions in it
// ---------------------------------------------------------------------------

#[test]
fn most_of_what_it_throws_can_be_answered_on_sight() {
    // All of it and the fight is a metronome; none of it and it is
    // memorisation. The one move below the reaction threshold has a positional
    // answer instead -- do not stand in front of it -- which is a different
    // kind of decision and worth having exactly one of.
    let moves_answerable = hunts()[0].reactable_moves();
    let damaging = hunts()[0].damaging_moves();
    assert!(
        moves_answerable * 2 > damaging,
        "only {moves_answerable} of {damaging} damaging moves can be reacted to. {REACTION_NOTE}"
    );
    assert!(
        moves_answerable < damaging,
        "every single move is reactable, so there is nothing that punishes \
         standing in the wrong place"
    );
}

#[test]
fn it_uses_its_whole_move_set() {
    // A move it never throws is a design fiction: it costs tuning, it costs a
    // line in the frame table, and it decides nothing.
    let mut ever = [0u32; MOVES];
    for report in hunts() {
        for (slot, count) in report.starts.iter().enumerate() {
            ever[slot] += count;
        }
    }
    let missing: Vec<&str> = (0..MOVES)
        .filter(|k| ever[*k] == 0)
        .map(|k| monster::MOVE_NAMES[k])
        .collect();
    assert!(
        missing.is_empty(),
        "never used across {} hunts: {missing:?}",
        SEEDS.len()
    );
}

#[test]
fn no_single_move_is_the_whole_creature() {
    // A monster with one answer is a puzzle you solve once. The favourite is
    // allowed to be the favourite -- the flank is a real strategy and the tail
    // sweep is what punishes it -- but not the majority.
    for (seed, report) in SEEDS.iter().zip(hunts()) {
        assert!(
            report.dominant_share() < 0.55,
            "seed {seed}: one move was {:.0}% of everything it did",
            report.dominant_share() * 100.0
        );
        assert!(
            report.longest_repeat <= 6,
            "seed {seed}: it threw the same move {} times running",
            report.longest_repeat
        );
    }
}

#[test]
fn an_opening_is_long_enough_to_actually_punish() {
    // A window shorter than the punish it is inviting is not an opening, it is
    // a tease -- and the player learns to stop trying, which is the worst thing
    // a fight can teach.
    let committed = moves::get(sim::Class::Champion, sim::state::SLOT_COMMITTED);
    let needed = committed.startup + committed.active;
    for (seed, report) in SEEDS.iter().zip(hunts()) {
        assert!(
            report.mean_opening() > needed as f32,
            "seed {seed}: average opening is {:.0} frames and the committed \
             attack needs {needed} to reach",
            report.mean_opening()
        );
    }
}

#[test]
fn it_is_never_just_standing_there() {
    for (seed, report) in SEEDS.iter().zip(hunts()) {
        assert!(
            report.idle_share() < 0.35,
            "seed {seed}: {:.0}% of the fight was the creature doing nothing",
            report.idle_share() * 100.0
        );
        assert!(
            report.moves_per_minute() > 20.0,
            "seed {seed}: only {:.0} moves a minute",
            report.moves_per_minute()
        );
    }
}

#[test]
fn it_is_dangerous_most_of_the_time_and_open_the_rest() {
    // **The shape asked for, 2026-09-25**: about four tenths of the fight in
    // which nothing lands before it answers, about a fifth in which anybody
    // can walk up and swing, and the rest open only to a poke or to a fast
    // way in -- a dash, a vault, a structure jump. These are bands rather
    // than the targets, because the targets are a feel and the bands are
    // what says the feel has not been lost: it is threatening for less than
    // two thirds of the fight, it has a walk-up window worth an eighth of it,
    // and the windows between are a real share rather than a rounding error.
    // See `docs/design/monsters.md` §"Threat modes".
    for (seed, report) in SEEDS.iter().zip(hunts()) {
        let threat = report.threat_share(Threat::Threatening);
        let walk = report.threat_share(Threat::WalkUp);
        let between = report.threat_share(Threat::PokeOnly) + report.threat_share(Threat::Skilled);
        assert!(
            (0.35..0.62).contains(&threat),
            "seed {seed}: threatening {:.0}% of the fight",
            threat * 100.0
        );
        assert!(
            walk > 0.12,
            "seed {seed}: safe to walk up on only {:.0}% of the fight",
            walk * 100.0
        );
        assert!(
            between > 0.2,
            "seed {seed}: open to a poke or a way in for only {:.0}% of the fight",
            between * 100.0
        );
    }
}

// ---------------------------------------------------------------------------
// The climb
// ---------------------------------------------------------------------------

#[test]
fn the_back_is_reachable_and_is_not_a_safe_room() {
    // Both halves matter. A back nobody can get onto makes the ridge
    // decorative; a back nobody is thrown off makes the rest of the fight
    // optional.
    let reports = hunts();
    let rides: u32 = reports.iter().map(|r| r.rides).sum();
    let thrown: u32 = reports.iter().map(|r| r.thrown).sum();
    let fled: u32 = reports.iter().map(|r| r.fled).sum();
    let ridge: u32 = reports.iter().map(|r| r.ridge_hits).sum();
    assert!(rides > 0, "nobody ever got on it");
    assert!(ridge > 0, "nobody ever reached the ridge");
    // A buck that is read and jumped ended the ride as surely as one that
    // threw the rider: both are the creature deciding when the ride is over.
    // Counting only throws made a hunter who answered every shake look like
    // one nothing could touch.
    assert!(
        (thrown + fled) * 4 > rides,
        "only {thrown} of {rides} rides ended in a buck and {fled} were left \
         under one -- the back is a place to stand rather than a wager"
    );
    for report in &reports {
        assert!(
            report.ride_share() < 0.8,
            "{:.0}% of the fight was spent standing on the creature",
            report.ride_share() * 100.0
        );
    }
}

#[test]
fn a_ride_is_long_enough_to_do_something_with() {
    // Long enough to cross the back and swing. Shorter than that and climbing
    // is a way to lose health rather than a way to spend it.
    let reports = hunts();
    let frames: u32 = reports.iter().map(|r| r.ride_frames).sum();
    let rides: u32 = reports.iter().map(|r| r.rides.max(1)).sum();
    let mean = frames as f32 / rides as f32;
    let poke = moves::get(sim::Class::Champion, sim::state::SLOT_POKE);
    assert!(
        mean > poke.whiff_cost() as f32,
        "the average ride is {mean:.0} frames and a poke alone costs {}",
        poke.whiff_cost()
    );
}

#[test]
fn breaking_its_poise_is_something_that_happens() {
    // The topple is what the climb is *for*. If it never fires, the ridge is a
    // damage multiplier rather than a goal, and the fight has no shape.
    let topples: u32 = hunts().iter().map(|r| r.topples).sum();
    assert!(
        topples > 0,
        "nobody put it on the ground across {} whole hunts",
        SEEDS.len()
    );
}

// ---------------------------------------------------------------------------
// Was it fair
// ---------------------------------------------------------------------------

#[test]
fn nothing_hits_from_somewhere_it_could_not_be_read() {
    // **The one that matters.** A monster can score well on every other line
    // here and still feel cheap, and when it does it will be because of damage
    // the player had no way to avoid: too fast to answer on sight, and reaching
    // somewhere they had no reason to think was inside it.
    for (seed, report) in SEEDS.iter().zip(hunts()) {
        assert_eq!(
            report.unanswerable, 0,
            "seed {seed}: {} hits landed from outside the move's own reach, too \
             fast to react to",
            report.unanswerable
        );
    }
}

#[test]
fn the_hunt_reaches_a_conclusion() {
    // An unresolved hunt is its own failure. It means neither side can finish
    // the other, which reads as a grind long before it reads as a challenge.
    for (seed, report) in SEEDS.iter().zip(hunts()) {
        assert!(
            !matches!(report.outcome, Outcome::Unresolved),
            "seed {seed}: ten minutes and nobody won"
        );
    }
}

#[test]
fn the_fight_is_neither_free_nor_hopeless() {
    // A scripted hunter with a fixed reaction delay and no adaptation is a
    // mediocre player. A creature that always beats one is too hard for anyone
    // to learn against; one that never does is not a monster.
    //
    // **Over eighteen hunts rather than the six the rest use.** A win rate is
    // the one measure here that is a coin flip per hunt: at the four in ten
    // this fight sits at since 2026-09-25, six losses in a row happen about
    // one time in twenty, and they did, the day the hunt's opening moved by
    // three seconds and nothing about the difficulty changed -- thirty hunts
    // either side of it won thirteen. Eighteen puts that chance near one in
    // ten thousand.
    let won = WIN_RATE_SEEDS
        .iter()
        .map(|s| play([sim::Class::Champion; MAX_PLAYERS], 1, BUDGET, *s))
        .filter(|r| matches!(r.outcome, Outcome::Killed(_)))
        .count();
    assert!(
        won > 0,
        "the scripted hunter lost every one of {} hunts",
        WIN_RATE_SEEDS.len()
    );
    assert!(
        won < WIN_RATE_SEEDS.len(),
        "the scripted hunter won every one of {} hunts, and it is not a good \
         player",
        WIN_RATE_SEEDS.len()
    );
}

/// The six the rest of the file uses, and twelve more. See
/// `the_fight_is_neither_free_nor_hopeless`.
const WIN_RATE_SEEDS: [u32; 18] = [
    1,
    7,
    101,
    2_222,
    60_013,
    0x2545_F491,
    3,
    11,
    42,
    999,
    31_337,
    7_777,
    12_345,
    65_537,
    271_828,
    314_159,
    1_618_033,
    8_675_309,
];

#[test]
fn the_fight_uses_the_arena() {
    // Two things standing still and hitting each other is a different game.
    for (seed, report) in SEEDS.iter().zip(hunts()) {
        assert!(
            report.spread.to_f32_for_render() > 4.0,
            "seed {seed}: the whole fight happened inside {:?} metres",
            report.spread
        );
    }
}
