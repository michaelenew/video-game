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
use sim::course;
use sim::coursecheck;
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

/// Which hops each class clears, from the course fixtures, every one replayed.
fn cleared() -> Vec<(String, Class, Vec<bool>)> {
    let text = include_str!("fixtures/courses.txt");
    let mut out: Vec<(String, Class, Vec<bool>)> = Vec::new();
    for (head, prog) in lines(text) {
        let slug = head[0].to_string();
        let from: usize = head[1].parse().unwrap();
        let hops: usize = head[2].parse().unwrap();
        let class = class_named(head[3]);
        let c = course::all()
            .find(|c| c.arena().slug() == slug)
            .unwrap_or_else(|| panic!("no course {slug}"));
        let st = coursecheck::stage(c, class, from, hops);
        assert!(
            search::run(&st, &prog).landed,
            "{slug} hop {} for the {}: {prog} no longer lands",
            from + 1,
            class.name()
        );
        let row = match out.iter_mut().find(|r| r.0 == slug && r.1 == class) {
            Some(r) => r,
            None => {
                out.push((slug.clone(), class, vec![false; c.route.len() - 1]));
                out.last_mut().unwrap()
            }
        };
        for h in from..from + hops {
            row.2[h] = true;
        }
    }
    out
}

fn finishes(rows: &[(String, Class, Vec<bool>)], slug: &str, class: Class) -> bool {
    rows.iter()
        .any(|r| r.0 == slug && r.1 == class && r.2.iter().all(|c| *c))
}

#[test]
fn every_course_line_lands_and_the_routes_are_what_the_document_says() {
    let rows = cleared();
    for class in [
        Class::ShadowReaver,
        Class::Elementalist,
        Class::BloodMage,
        Class::DualMage,
        Class::Champion,
    ] {
        for slug in ["stair", "causeway", "gallery", "narrows", "sill"] {
            assert!(
                finishes(&rows, slug, class),
                "the {} does not finish {slug}",
                class.name()
            );
        }
    }
    assert!(finishes(&rows, "spire", Class::Elementalist));
    assert!(finishes(&rows, "eyrie", Class::Elementalist));
    assert!(finishes(&rows, "gulf", Class::ShadowReaver));
}
