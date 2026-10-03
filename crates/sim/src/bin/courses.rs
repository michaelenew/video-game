//! **The jump courses, measured**: every hop of every course, for every class
//! the courses are for, played in the sim (`sim::coursecheck`).
//!
//!     cargo run --release -p sim --bin courses
//!     cargo run --release -p sim --bin courses -- climb drift
//!
//! For each hop: what was built (the gap, the rise), and for each class the
//! **timing window** -- how many of sixteen takeoff frames a plain running
//! jump lands from (`J`), how many airdodge frames land at the best takeoff
//! (`D`) -- the **distance margin** -- how much wider the gap could be and
//! still be landed by the running jump, plain/airdodged, off the envelope's
//! trajectories -- and which of the class's own tools also clear it. Then the
//! whole course, each hop with the first technique that clears it: the
//! measured matrix in `docs/design/courses.md` §5.
//!
//! Integer arithmetic throughout: this file lives under `crates/sim/src`, and
//! the no-floats guard covers it.

use sim::arena::cm;
use sim::class::Class;
use sim::course;
use sim::coursecheck as c;
use sim::envelope::{self as e, Extra};
use sim::{Fx, TICK_HZ};

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

fn secs(frames: u32) -> String {
    let tenths = frames * 10 / TICK_HZ;
    format!("{}.{}s", tenths / 10, tenths % 10)
}

fn main() {
    let asked: Vec<String> = std::env::args().skip(1).collect();
    let wanted = |name: &str| asked.is_empty() || asked.iter().any(|a| a == name);
    // The trajectories every distance margin is read from, once per class.
    let paths: Vec<(Vec<e::Path>, Vec<e::Path>)> = CLASSES
        .iter()
        .map(|class| {
            (
                vec![e::off_the_edge(*class, Extra::Nothing)],
                e::airdodge_paths(*class),
            )
        })
        .collect();
    println!("The jump courses, measured. J = takeoff frames of 16 a plain jump lands from;");
    println!(
        "D = airdodge frames that land; +a/+b = metres the gap could widen, plain/airdodged.\n"
    );
    for course in course::all() {
        let slug = course.arena().slug();
        if !wanted(&slug) {
            continue;
        }
        println!(
            "{} -- {} (`--arena {}`){}",
            course.name,
            course.tier.name(),
            slug,
            course.for_class.map_or(String::new(), |k| format!(
                ", built against the {}",
                k.name()
            ))
        );
        print!("  {:<36}", "hop");
        for class in CLASSES {
            print!(" {:>26}", short(class));
        }
        println!();
        for from in 0..course.route.len() - 1 {
            let step = course.route[from + 1];
            print!(
                "  {:<36}",
                format!(
                    "{}. {} ({}m, {}m)",
                    from + 1,
                    step.ask,
                    e::metres(cm(step.gap)),
                    e::metres(cm(step.rise))
                )
            );
            for (k, class) in CLASSES.iter().enumerate() {
                let w = c::standing_on(course, *class, from);
                let v = c::judge(&w, course, from, *class);
                let (plain, dodged) = &paths[k];
                let rise = cm(step.rise);
                let gap = cm(step.gap);
                let spare = |p: &[e::Path]| {
                    e::widest(p, rise).map_or("-".to_string(), |far| e::metres(far.sub(gap)))
                };
                let mut tools: Vec<String> = Vec::new();
                for t in &v.tools {
                    let n = v.tools.iter().filter(|o| o.family() == t.family()).count();
                    let named = format!("{}x{n}", t.family());
                    if !tools.contains(&named) {
                        tools.push(named);
                    }
                }
                let cell = if v.cleared() {
                    format!(
                        "J{} D{} {}/{}{}",
                        v.jump_window,
                        v.dodge_window,
                        spare(plain),
                        spare(dodged),
                        if tools.is_empty() {
                            String::new()
                        } else {
                            format!(" {}", tools.join("+"))
                        }
                    )
                } else {
                    format!("NO {}/{}", spare(plain), spare(dodged))
                };
                print!(" {:>26}", cell);
            }
            println!();
        }
        println!("  whole course, each hop with the first thing that clears it:");
        for class in CLASSES {
            let (done, frames, finished) = c::run(course, class);
            let how: Vec<String> = done.iter().map(|(_, t)| t.family().to_string()).collect();
            println!(
                "    {:<9} {:<9} {:>6}  {}",
                short(class),
                if finished { "finishes" } else { "stuck" },
                if finished {
                    secs(frames)
                } else {
                    format!("at {}", done.len() + 1)
                },
                how.join(", ")
            );
        }
        println!();
    }
    let _ = Fx::ZERO;
}
