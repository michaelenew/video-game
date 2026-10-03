//! The jump courses (`docs/design/courses.md`): their tables are well formed,
//! a fall stands you back on your last checkpoint, the clock stops at the
//! finish -- and **every route the document promises is played**: the two
//! hard courses by all five classes they are for, and each barely-possible
//! course by the class it was built against. A route that stops working when
//! somebody retunes a jump is the point: the course was built against those
//! numbers, and the document says which.
//!
//! The routes are `sim::coursecheck::run`: scripted input, a hop at a time,
//! each with the first technique that clears it.

use sim::arena::ArenaId;
use sim::class::Class;
use sim::course::{self, Run, Tier};
use sim::coursecheck;
use sim::input::{Destination, Travel};
use sim::state::MAX_PLAYERS;
use sim::{Fx, Input, V3, World};

/// The five classes the courses are for; the Bulwark is out of this by the
/// owner's choice (courses.md §0).
const FIVE: [Class; 5] = [
    Class::ShadowReaver,
    Class::Elementalist,
    Class::BloodMage,
    Class::DualMage,
    Class::Champion,
];

fn named(name: &str) -> &'static course::Course {
    course::all()
        .find(|c| c.arena().slug() == name)
        .unwrap_or_else(|| panic!("no course called {name}"))
}

#[test]
fn every_course_is_well_formed() {
    let mut tiers = Vec::new();
    for c in course::all() {
        let a = c.arena();
        assert!(
            c.route.len() >= 2,
            "{}: a route needs a start and a finish",
            c.name
        );
        assert!(c.gate_count() >= 2, "{}", c.name);
        assert_eq!(course::of(a.id).map(|x| x.name), Some(c.name));
        // The start stands on the floor, where the marks are; every other
        // island hangs in the air over the pit.
        assert!(
            !c.top(0).hangs(),
            "{}: the start must stand on the floor",
            c.name
        );
        for i in 1..c.route.len() {
            assert!(c.top(i).hangs(), "{}: step {i} stands on the floor", c.name);
        }
        let pit = sim::arena::cm(c.pit);
        for i in 0..c.route.len() {
            assert!(
                c.top(i).max.y.raw() > pit.raw(),
                "{}: step {i}'s top is in the pit",
                c.name
            );
        }
        // The marks are on the start.
        for mark in a.spawns.versus {
            assert!(
                c.gate(0)
                    .holds(V3::new(mark.at.x, c.top(0).max.y, mark.at.z))
            );
        }
        tiers.push(c.tier);
    }
    // Ordered by tier, as the key steps through them, and more than one of
    // each.
    let mut sorted = tiers.clone();
    sorted.sort();
    assert_eq!(tiers, sorted, "the courses are listed out of tier order");
    for t in [Tier::Easy, Tier::Hard, Tier::Edge] {
        assert!(tiers.iter().filter(|x| **x == t).count() >= 2, "{t:?}");
    }
}

#[test]
fn a_fall_stands_you_on_your_last_checkpoint_and_counts() {
    let c = named("climb");
    let mut w = World::versus_in([Class::Champion; MAX_PLAYERS], c.arena);
    // Reach the first checkpoint: stand on it.
    let check = c.gate(1);
    w.players[0].pos = check.stand();
    w.advance([Input::default(); MAX_PLAYERS]);
    assert_eq!(
        w.course[0].reached, 1,
        "standing on a checkpoint did not reach it"
    );
    assert!(
        w.course[0].started > 0,
        "leaving the start did not start the clock"
    );
    // And fall into the pit.
    w.players[0].pos = V3::new(check.stand().x, Fx::ZERO, check.stand().z);
    w.players[0].health -= 100;
    w.advance([Input::default(); MAX_PLAYERS]);
    let p = w.players[0];
    assert_eq!(w.course[0].falls, 1);
    assert_eq!(p.pos.x, check.stand().x);
    assert_eq!(p.pos.y, check.stand().y, "not stood back on the checkpoint");
    assert_eq!(
        p.health,
        p.full_health(),
        "a fall is a fresh start, not a death"
    );
    assert_eq!(w.course[0].reached, 1, "the fall cost the checkpoint");
    // A few frames later she is standing there, not falling through it.
    for _ in 0..10 {
        w.advance([Input::default(); MAX_PLAYERS]);
    }
    assert!(w.players[0].grounded);
    assert_eq!(w.players[0].pos.y, check.stand().y);
}

#[test]
fn the_clock_runs_from_the_start_and_stops_at_the_finish() {
    let c = named("stair");
    let mut w = World::versus_in([Class::Champion; MAX_PLAYERS], c.arena);
    w.advance([Input::default(); MAX_PLAYERS]);
    assert_eq!(w.course[0].clock(w.frame), 0, "the clock ran on the start");
    w.players[0].pos = c.gate(1).stand();
    for _ in 0..30 {
        w.advance([Input::default(); MAX_PLAYERS]);
    }
    w.players[0].pos = c.gate(c.finish()).stand();
    w.advance([Input::default(); MAX_PLAYERS]);
    let run: Run = w.course[0];
    assert!(run.finished());
    let time = run.time;
    for _ in 0..30 {
        w.advance([Input::default(); MAX_PLAYERS]);
    }
    assert_eq!(w.course[0].time, time, "the clock ran past the finish");
    assert_eq!(w.course[0].clock(w.frame), time);
}

#[test]
fn a_course_is_a_trip_on_the_wire_and_the_key_steps_through_them_all() {
    let first = course::all().next().expect("no courses");
    let byte = Travel::arena(first.arena);
    assert_eq!(byte.destination(), Some(Destination::Arena(first.arena)));
    let mut w = World::with_classes([Class::Champion; MAX_PLAYERS]);
    let mut ask = [Input::default(); MAX_PLAYERS];
    ask[0] = ask[0].travelling(byte);
    w.advance(ask);
    assert_eq!(w.arena, first.arena);
    assert!(!w.hunting());
    // Stepping visits every course and comes back round.
    let mut at = first.arena;
    let mut seen = Vec::new();
    for _ in 0..course::all().count() {
        seen.push(at);
        at = course::after(at).arena;
    }
    assert_eq!(at, first.arena);
    seen.sort_by_key(|a| a.0);
    seen.dedup();
    assert_eq!(seen.len(), course::all().count());
    // From somewhere that is not a course, the first.
    assert_eq!(course::after(ArenaId::PROVING_GROUND).arena, first.arena);
    // An unregistered arena is no trip.
    let mut w = World::with_classes([Class::Champion; MAX_PLAYERS]);
    let mut ask = [Input::default(); MAX_PLAYERS];
    ask[0] = ask[0].travelling(Travel::arena(ArenaId(60)));
    w.advance(ask);
    assert_eq!(w.arena, ArenaId::PROVING_GROUND);
}

fn finishes(name: &str, class: Class) {
    let c = named(name);
    let (done, frames, finished) = coursecheck::run(c, class);
    let how: Vec<String> = done.iter().map(|(_, t)| t.name()).collect();
    assert!(
        finished,
        "{} did not finish {}: stuck after {:?}",
        class.name(),
        c.name,
        how
    );
    assert!(frames > 0);
}

/// **The main route is hard and everybody finishes it**: the owner's goal
/// for these courses (courses.md §0). Every class, on its jump and airdodge
/// and whatever else it has.
#[test]
fn every_class_finishes_the_climb() {
    for class in FIVE {
        finishes("climb", class);
    }
}

#[test]
fn every_class_finishes_the_drift() {
    for class in FIVE {
        finishes("drift", class);
    }
}

/// **The Spire is possible**, for the class it was built against: a double
/// stone jump off the launch, 32 m up onto the rookery.
#[test]
fn the_elementalist_climbs_the_spire() {
    finishes("spire", Class::Elementalist);
}

/// **The Gulf is possible**, for the class it was built against: a shadow
/// sent onto a stone a metre across, the dash to it and the dash jump off it,
/// and the last island by the shadow alone.
#[test]
fn the_reaver_crosses_the_gulf() {
    finishes("gulf", Class::ShadowReaver);
}
