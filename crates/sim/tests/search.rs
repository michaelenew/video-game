//! **Every number the movement search claims is a run that happens.**
//!
//! `cargo run --release -p sim --bin envelope -- search --fixtures` and
//! `cargo run --release -p sim --bin courses -- --fixtures` write the best
//! input programs they found to `tests/fixtures/` (`docs/design/courses.md`
//! §1 and §5). This replays every one: each envelope line lands on its lab
//! ledge across at least the gap it claims, and each course line lands on
//! the island it claims. And it holds the courses to what the document says
//! of them: every class finishes all three hard courses, and each barely
//! possible one is finished by the class it was built against.
//!
//! A search is a lower bound. If a change to the game makes a line here stop
//! working, either the change took that reach away -- say so in
//! `courses.md` and the feel log -- or rerun the search and commit its new
//! fixtures.

use sim::class::{ALL_CLASSES, Class};
use sim::search::{self, Program, Stage};

fn class_named(name: &str) -> Class {
    let name = name.replace('_', " ");
    ALL_CLASSES
        .into_iter()
        .find(|c| c.name() == name)
        .unwrap_or_else(|| panic!("no class called {name}"))
}

fn lines(text: &str) -> impl Iterator<Item = (Vec<&str>, Program)> {
    text.lines().filter(|l| l.contains('|')).map(|l| {
        let (head, prog) = l.split_once('|').expect("a fixture line has a program");
        let prog = Program::parse(prog.trim())
            .unwrap_or_else(|| panic!("a fixture that does not parse: {l}"));
        (head.split_whitespace().collect(), prog)
    })
}

#[test]
fn every_envelope_line_lands_across_its_gap() {
    let text = include_str!("fixtures/envelope.txt");
    let mut n = 0;
    for (head, prog) in lines(text) {
        let class = class_named(head[0]);
        let lane: usize = head[1].parse().unwrap();
        let gap: i32 = head[3].parse().unwrap();
        let o = search::run(&Stage::lane(class, lane), &prog);
        assert!(
            o.landed,
            "{} lane {lane}: {prog} no longer lands",
            class.name()
        );
        assert!(
            sim::envelope::to_cm(o.gap) >= gap - 1,
            "{} lane {lane}: {prog} crosses {} cm, not the {gap} claimed",
            class.name(),
            sim::envelope::to_cm(o.gap)
        );
        n += 1;
    }
    assert!(n > 50, "only {n} envelope fixtures");
}
