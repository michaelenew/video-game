//! **The jump courses, measured**: every hop of every course, for every class
//! the courses are for, searched (`sim::coursecheck`, `sim::search`).
//!
//!     cargo run --release -p sim --bin courses
//!     cargo run --release -p sim --bin courses -- climb drift --budget 1500
//!     cargo run --release -p sim --bin courses -- --fixtures 2> crates/sim/tests/fixtures/courses.txt
//!
//! For each hop and class: `S` and the window if the shared blocks land it
//! (the jump, the airdodge, aerials, the strafe), otherwise the whole kit's
//! tools and window, otherwise `NO`. The window is how many of 31
//! frames the tightest input of the plainest line found can move and still
//! land. Then, for each class, whether every hop is cleared (a hop nothing
//! clears alone may be crossed with the one before it, over a stepping stone).
//! A search is a lower bound.
//!
//! Integer arithmetic throughout: this file lives under `crates/sim/src`, and
//! the no-floats guard covers it.

use sim::arena::cm;
use sim::class::Class;
use sim::course;
use sim::coursecheck as c;
use sim::envelope as e;
use sim::search::Kit;

/// The five classes the courses are for (the Bulwark is out by the owner's
/// choice: `docs/design/courses.md` §0).
const CLASSES: [Class; 5] = [
    Class::ShadowReaver,
    Class::Elementalist,
    Class::BloodMage,
    Class::DualMage,
    Class::Champion,
];

fn short(class: Class) -> &'static str {
    match class {
        Class::ShadowReaver => "Reaver",
        Class::Elementalist => "Element.",
        Class::BloodMage => "Blood",
        Class::DualMage => "Dual",
        Class::Champion => "Champion",
        Class::Bulwark => "Bulwark",
    }
}

fn budget() -> usize {
    let args: Vec<String> = std::env::args().collect();
    args.iter()
        .position(|a| a == "--budget")
        .and_then(|i| args.get(i + 1))
        .and_then(|n| n.parse().ok())
        .unwrap_or(800)
}

/// `--bench`: every lane of the bench arena, every class, the most forgiving
/// line's window -- the shared blocks, and the whole kit.
fn bench(budget: usize) {
    use sim::arena::{ArenaId, bench};
    let solids = ArenaId::BENCH.get().solids();
    println!("bench -- the most forgiving line's tightest window, of 31, whole kit");
    print!("  {:<22}", "hop");
    for class in CLASSES {
        print!(" {:>10}", short(class));
    }
    println!();
    for (name, a, b) in bench::LANES {
        print!("  {:<22}", name);
        for class in CLASSES {
            let st =
                sim::search::Stage::hop(ArenaId::BENCH, class, solids[a], solids[b], solids[b]);
            let seed = (class as u64) << 8 | b as u64;
            let w = |kit| {
                c::loosest(&st, kit, budget, seed).map_or("-".to_string(), |l| l.window.to_string())
            };
            print!(" {:>10}", w(Kit::Full));
        }
        println!();
    }
}

fn main() {
    if std::env::args().any(|a| a == "--bench") {
        bench(budget());
        return;
    }
    let asked: Vec<String> = std::env::args()
        .skip(1)
        .filter(|a| !a.starts_with("--") && a.parse::<usize>().is_err())
        .collect();
    let wanted = |name: &str| asked.is_empty() || asked.iter().any(|a| a == name);
    let fixtures = std::env::args().any(|a| a == "--fixtures");
    let budget = budget();
    {
        println!(
            "The jump courses, searched ({budget} runs per search; a search is a lower bound)."
        );
        println!("Each cell: the most forgiving line found -- how many of 31 frames its tightest");
        println!("input can move and still land -- and any tool past the jump, airdodge and");
        println!("strafe it uses; NO: nothing found.\n");
    }
    for course in course::all() {
        let slug = course.arena().slug();
        if !wanted(&slug) {
            continue;
        }
        {
            println!(
                "{} -- {} (`--arena {}`)",
                course.name,
                course.tier.name(),
                slug
            );
            print!("  {:<40}", "hop");
            for class in CLASSES {
                print!(" {:>22}", short(class));
            }
            println!();
        }
        let hops = course.route.len() - 1;
        // cleared[class][hop]: by itself, or as the second of a pair.
        let mut cleared = vec![vec![false; hops]; CLASSES.len()];
        let mut cells = vec![vec![String::new(); CLASSES.len()]; hops];
        for from in 0..hops {
            for (k, class) in CLASSES.iter().enumerate() {
                let seed = (*class as u64) << 24 | (course.arena.0 as u64) << 8 | from as u64;
                let st = c::stage(course, *class, from, 1);
                // A miss is often the search's and not the class's: once more,
                // harder, before calling it.
                let line = c::loosest(&st, Kit::Full, budget, seed)
                    .or_else(|| c::loosest(&st, Kit::Full, budget * 3, seed.rotate_left(17)));
                let cell = match &line {
                    Some(l) => {
                        let tools: Vec<&str> = c::uses(&l.program)
                            .into_iter()
                            .filter(|t| !matches!(*t, "jump" | "strafe"))
                            .collect();
                        if tools.is_empty() {
                            format!("{}", l.window)
                        } else {
                            format!("{} {}", l.window, tools.join("+"))
                        }
                    }
                    None => "NO".to_string(),
                };
                if let (true, Some(l)) = (fixtures, &line) {
                    eprintln!(
                        "{slug} {from} 1 {} full | {}",
                        class.name().replace(' ', "_"),
                        l.program
                    );
                }
                if line.is_some() {
                    cleared[k][from] = true;
                }
                cells[from][k] = cell;
            }
        }
        // A hop nothing clears alone: the pair from the step before.
        for (k, class) in CLASSES.iter().enumerate() {
            for from in 1..hops {
                if cleared[k][from] {
                    continue;
                }
                let seed =
                    (*class as u64) << 24 | (course.arena.0 as u64) << 8 | 0x80 | from as u64;
                let pair = c::loosest(
                    &c::stage(course, *class, from - 1, 2),
                    Kit::Full,
                    budget,
                    seed,
                );
                if let Some(l) = pair {
                    cleared[k][from] = true;
                    cells[from][k] =
                        format!("pair: {} {}", c::uses(&l.program).join("+"), l.window);
                    if fixtures {
                        eprintln!(
                            "{slug} {} 2 {} full | {}",
                            from - 1,
                            class.name().replace(' ', "_"),
                            l.program
                        );
                    }
                }
            }
        }
        for (from, row) in cells.iter().enumerate() {
            let step = course.route[from + 1];
            print!(
                "  {:<40}",
                format!(
                    "{}. {} ({}m, {}m)",
                    from + 1,
                    step.ask,
                    e::metres(cm(step.gap)),
                    e::metres(cm(step.rise))
                )
            );
            for cell in row {
                print!(" {:>22}", cell);
            }
            println!();
        }
        print!("  {:<40}", "finishes");
        for row in &cleared {
            let done = row.iter().all(|c| *c);
            let stuck = row.iter().position(|c| !c).map_or(0, |i| i + 1);
            print!(
                " {:>22}",
                if done {
                    "yes".to_string()
                } else {
                    format!("no, hop {stuck}")
                }
            );
        }
        println!("\n");
    }
}
