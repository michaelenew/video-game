//! **Every fight, measured in reaches and jumps** (`sim::fightspace`,
//! `docs/design/fight-spaces.md`).
//!
//!     cargo run --release -p sim --bin fightspace
//!
//! One line a place: its size in reaches, how far its starts are from its
//! middle, how much of where you can stand is out of reach of the middle,
//! its levels a jump and a half apart, and what in it moves you. The
//! versus stages first, then every other place a fight is fought alone in.

use sim::arena::{self, Arena};
use sim::class::MOVERS;
use sim::fightspace::{self, Measure};
use sim::fixed::Fx;

fn tenths(v: Fx) -> String {
    let t = v.mul(Fx::from_int(10)).to_int();
    format!("{}.{}", t / 10, t.abs() % 10)
}

fn line(a: &Arena, m: &Measure, r: Fx, j: Fx) {
    let tiers: Vec<String> = m.tiers.iter().map(|&h| tenths(h)).collect();
    println!(
        "{:<16} {:>4}x{:<4}m  {:>3}x{:<3}R  start {:>4}m {:>3}R  beyond R {:>3}%  farthest {:>3}R  levels {} [{}] top {:>3}J  rings {:>2}/{:>2}/{:>2}  vents {} drifts {} hanging {:>2} ways {:>2} ({} to the point)",
        a.name,
        m.length.to_int(),
        m.width.to_int(),
        tenths(m.length.div(r)),
        tenths(m.width.div(r)),
        m.start_to_centre.to_int(),
        tenths(m.start_to_centre.div(r)),
        m.beyond_reach,
        tenths(m.farthest.div(r)),
        m.tiers.len(),
        tiers.join(" "),
        tenths(m.top().div(j)),
        m.rings[0],
        m.rings[1],
        m.rings[2],
        m.vents,
        m.drifts,
        m.hanging,
        m.ways,
        m.ways_to_centre,
    );
}

fn main() {
    let (r, j) = (fightspace::reach(), fightspace::jump());
    println!(
        "reach R = {} m: the movers' longest pointed reaches are",
        tenths(r)
    );
    for c in MOVERS {
        println!(
            "  {:<14} {} m  (full jump {} m)",
            c.name(),
            tenths(fightspace::pointed_reach(c)),
            tenths(fightspace::full_jump(c))
        );
    }
    println!(
        "jump J = {} m; a level is a jump and a half, {} m, over the one below\n",
        tenths(j),
        tenths(j.add(j.div(Fx::from_int(2))))
    );
    println!("-- versus stages --");
    for s in sim::versus::STAGES {
        let a = s.arena.get();
        line(a, &fightspace::measure(a, r, j), r, j);
    }
    println!("\n-- every other fight fought alone --");
    for a in arena::all() {
        let fought = a.creature.is_some() || a.id == arena::ArenaId::PROVING_GROUND;
        if fought && sim::versus::of(a.id).is_none() {
            line(a, &fightspace::measure(a, r, j), r, j);
        }
    }
}
