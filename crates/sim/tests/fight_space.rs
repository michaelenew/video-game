//! **The fight-space rules** (`docs/design/fight-spaces.md` §4), held for
//! every versus stage: a place measured in the game's own units -- a reach,
//! the second-longest of the movers' pointed reaches, and a jump, the
//! shortest of their full held jumps (`sim::fightspace`).
//!
//! The owner, 2026-11-02: "the koth levels are just tiny relative to each
//! class's mobility options, as are the 1v1 stages". A map that fails here
//! is a map where everybody stands in the middle: nowhere out of reach,
//! one level, nothing that moves you.

use sim::fightspace::{self, Measure};
use sim::fixed::Fx;
use sim::versus::{self, Kind};

fn of(n: i32, d: i32, unit: Fx) -> Fx {
    unit.mul(Fx::ratio(n, d))
}

fn at_least(what: &str, name: &str, got: Fx, want: Fx, fails: &mut Vec<String>) {
    if got.raw() < want.raw() {
        fails.push(format!(
            "{name}: {what} is {:.1} m, the rule wants {:.1}",
            got.raw() as f64 / 65536.0,
            want.raw() as f64 / 65536.0
        ));
    }
}

fn hill_map(m: &Measure, name: &str, r: Fx, j: Fx, fails: &mut Vec<String>) {
    at_least("its length", name, m.length, of(5, 1, r), fails);
    at_least("its width", name, m.width, of(7, 2, r), fails);
    at_least(
        "start to the point",
        name,
        m.start_to_centre,
        of(11, 5, r),
        fails,
    );
    if m.tiers.len() < 3 {
        fails.push(format!(
            "{name}: {} levels, the rule wants 3",
            m.tiers.len()
        ));
    }
    at_least("its top level", name, m.top(), of(7, 2, j), fails);
    let all: usize = m.rings.iter().sum();
    let [core, contest, away] = m.rings;
    if core * 5 > all || contest * 3 < all || away * 4 < all {
        fails.push(format!(
            "{name}: structure core/contest/out of reach is {core}/{contest}/{away} of {all}; \
             the rule wants at most a fifth, at least a third, at least a quarter"
        ));
    }
    if m.ways_to_centre < 3 {
        fails.push(format!(
            "{name}: {} ways to the point, the rule wants 3",
            m.ways_to_centre
        ));
    }
}

fn duel_stage(m: &Measure, name: &str, r: Fx, j: Fx, fails: &mut Vec<String>) {
    at_least(
        "its pieces' span",
        name,
        m.span.0.max(m.span.1),
        of(9, 5, r),
        fails,
    );
    at_least("start to start", name, m.starts_apart, of(4, 5, r), fails);
    if m.tiers.len() < 2 {
        fails.push(format!(
            "{name}: {} levels, the rule wants 2",
            m.tiers.len()
        ));
    }
    at_least("its top level", name, m.top(), of(2, 1, j), fails);
}

#[test]
fn every_versus_stage_has_room_to_fight() {
    let (r, j) = (fightspace::reach(), fightspace::jump());
    let mut fails = Vec::new();
    for s in versus::STAGES {
        let a = s.arena.get();
        let m = fightspace::measure(a, r, j);
        match s.kind {
            Kind::Hill => hill_map(&m, a.name, r, j, &mut fails),
            Kind::Duel => duel_stage(&m, a.name, r, j, &mut fails),
        }
        if m.vents + m.drifts == 0 {
            fails.push(format!(
                "{}: nothing moves you, no updraft and no drift",
                a.name
            ));
        }
    }
    assert!(fails.is_empty(), "\n{}", fails.join("\n"));
}

/// The yardsticks are what the rules were written against: if a kit change
/// moves them a long way, the rules want reading again, not just the maps.
#[test]
fn the_yardsticks_are_what_the_rules_were_written_against() {
    let (r, j) = (fightspace::reach(), fightspace::jump());
    assert!(
        r.raw() >= Fx::from_int(20).raw() && r.raw() <= Fx::from_int(30).raw(),
        "a reach is {r:?}: docs/design/fight-spaces.md §1 wants reading again"
    );
    assert!(
        j.raw() >= Fx::from_int(3).raw() && j.raw() <= Fx::from_int(5).raw(),
        "a jump is {j:?}: docs/design/fight-spaces.md §1 wants reading again"
    );
}
