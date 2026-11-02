//! **The camera check from the command line** (`view::camcheck`,
//! `docs/design/level-design.md`, "The play check"):
//!
//! ```text
//! cargo run -p view --example camcheck -- <arena> "x,z,from; x,z,from; ..."
//! ```
//!
//! Walks a fighter along the stops (metres; `from` is the height to look
//! down from for what is underfoot, as `SHOT_AT`'s third number) with the
//! fight's camera following, turns the view round at each stop, and prints
//! what it found. Exits non-zero when the place fails the bar.

use sim::{Class, World};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let (Some(name), Some(route)) = (args.first(), args.get(1)) else {
        eprintln!("usage: camcheck <arena> \"x,z,from; x,z,from; ...\"");
        std::process::exit(2);
    };
    let Some(arena) = sim::arena::named(name) else {
        eprintln!("no arena called {name}");
        std::process::exit(2);
    };
    let stops: Vec<[f32; 3]> = route
        .split(';')
        .filter(|s| !s.trim().is_empty())
        .map(|s| {
            let v: Vec<f32> = s.split(',').filter_map(|n| n.trim().parse().ok()).collect();
            [v[0], v[1], v.get(2).copied().unwrap_or(1000.0)]
        })
        .collect();
    let world = World::versus_in([Class::Champion; sim::state::MAX_PLAYERS], arena.id);
    let report = view::camcheck::walk(&world.terrain(), &stops, &Default::default());
    print!("{}", report.text());
    if !report.passes() {
        std::process::exit(1);
    }
}
