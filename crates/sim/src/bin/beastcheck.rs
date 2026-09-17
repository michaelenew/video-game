//! What the creature actually measures, in the states that matter.
//!
//!     cargo run -p sim --bin beastcheck
//!
//! The climb is a geometry problem before it is a timing one -- can a jump
//! reach that, and from where -- and geometry arguments conducted in prose go
//! wrong. This prints the numbers: how high every surface you can stand on is,
//! standing and in each of the states that lower one, against how high a
//! fighter can actually jump from the floor and from an arena platform.
//!
//! Fixed point throughout, like everything else in this crate, and displayed
//! through the Oven's own formatter. A diagnostic doing its arithmetic in `f32`
//! would be reporting on a different simulation from the one that runs.
//!
//! `docs/design/monsters.md` §2 quotes this.

use sim::beast::{self, Clip};
use sim::fixed::Fx;
use sim::math::V3;
use sim::monster::{self, Doing, Monster};
use sim::oven::Unit;
use sim::state::MAX_PLAYERS;
use sim::tuning as t;

fn m(v: Fx) -> String {
    Unit::Fixed.show(v.raw())
}

/// Apex of a full hop above the feet, for one class.
///
/// **Measured by running the simulation**, not solved. The sustain window makes
/// the closed form wrong, and a diagnostic that re-derives the physics it is
/// reporting on is the same mistake as an overlay that rebuilds the geometry it
/// illustrates: it is confidently wrong exactly when you are relying on it.
fn jump_apex(class: sim::Class) -> Fx {
    let mut w = sim::World::with_classes([class; MAX_PLAYERS]);
    for _ in 0..40 {
        w.advance([sim::Input::default(); MAX_PLAYERS]);
    }
    let floor = w.players[0].pos.y;
    let mut best = floor;
    let held = sim::Input::default().with(sim::Input::SPACE);
    for _ in 0..240 {
        w.advance([held, sim::Input::default()]);
        best = best.max(w.players[0].pos.y);
    }
    best.sub(floor)
}

/// The world height of a part's top face, at the middle of it.
fn top(beast: &Monster, part: usize) -> Fx {
    let sh = monster::shape(part);
    let mid = |a: Fx, b: Fx| a.add(b).mul(Fx::ratio(1, 2));
    beast
        .rig()
        .part_to_world(
            part,
            V3::new(mid(sh.min.x, sh.max.x), sh.max.y, mid(sh.min.z, sh.max.z)),
        )
        .y
}

fn holding(doing: Doing) -> Monster {
    let mut beast = Monster::new();
    beast.doing = doing;
    beast
}

/// The lowest a part's top face gets at any point in a move, which is what
/// decides whether the move opens a way up.
fn lowest_through(kind: u8, part: usize) -> Fx {
    let a = monster::attack(kind);
    let mut best = Fx::MAX;
    for (which, span) in [(0u8, a.startup), (1, a.active), (2, a.recovery)] {
        for left in 0..span {
            let doing = match which {
                0 => Doing::Startup { kind, left },
                1 => Doing::Active { kind, left },
                _ => Doing::Recovery { kind, left },
            };
            best = best.min(top(&holding(doing), part));
        }
    }
    best
}

fn main() {
    // **Two jumps, not one.** This printed a single apex until 2026-09-17,
    // which was honest while the cast spanned four metres to seven and a half:
    // the shortest hop in the game got you most of the way up, so answering for
    // the heaviest class answered for everybody. The mobility pass took the
    // whole cast down and left a spread of better than two to one, so a single
    // number now hides the thing the climb turns on -- which classes can get up
    // there at all.
    let mut lowest = (sim::Class::Bulwark, Fx::MAX);
    let mut highest = (sim::Class::Bulwark, Fx::ZERO);
    for class in sim::class::ALL_CLASSES {
        let apex = jump_apex(class);
        if apex.raw() < lowest.1.raw() {
            lowest = (class, apex);
        }
        if apex.raw() > highest.1.raw() {
            highest = (class, apex);
        }
    }
    let (short, apex) = lowest;
    let (tall, best) = highest;
    let platform = sim::arena::WALL_HEIGHT;
    let from_platform = best.add(platform);
    println!(
        "a full hop reaches {} m off the floor ({}) to {} m ({})",
        m(apex),
        short.name(),
        m(best),
        tall.name()
    );
    println!(
        "the arena's platforms are {} m, so from one the best of them reaches {} m\n",
        m(platform),
        m(from_platform)
    );

    let reach = |h: Fx| {
        if h.raw() <= apex.raw() {
            "a standing jump"
        } else if h.raw() <= best.raw() {
            "a standing jump, the floatier classes"
        } else if h.raw() <= from_platform.raw() {
            "only from a platform"
        } else {
            "out of reach"
        }
    };

    let standing = holding(Doing::Prowl);
    println!("standing, the tops of the surfaces you can stand on:");
    for part in 0..monster::PARTS {
        if !beast::SHAPES[part].mountable {
            continue;
        }
        let h = top(&standing, part);
        println!(
            "  {:<14} {:>6} m   {}",
            monster::PART_NAMES[part],
            m(h),
            reach(h)
        );
    }

    println!("\nthe weak points, standing:");
    for part in [monster::RIDGE, monster::NAPE] {
        let sh = monster::shape(part);
        let high = top(&standing, part);
        println!(
            "  {:<14} {:>6} m at its foot, {:>6} m at its top, x{} damage",
            monster::PART_NAMES[part],
            m(high.sub(sh.max.y.sub(sh.min.y))),
            m(high),
            m(monster::vulnerability(part))
        );
    }

    println!("\nand what opens a way up (lowest the surface gets):");
    let route = |label: &str, h: Fx| {
        println!("  {label:<34} {:>6} m   {}", m(h), reach(h));
    };
    let stumbled = |front: bool, part: usize| {
        let mut best = Fx::MAX;
        for left in 0..t::stumble_frames() {
            best = best.min(top(&holding(Doing::Stumble { left, front }), part));
        }
        best
    };
    route(
        "the shoulders, stumbling",
        stumbled(true, monster::SHOULDERS),
    );
    route("the haunch, stumbling", stumbled(false, monster::HAUNCH));
    let mut lame = Monster::new();
    for leg in beast::LEGS {
        if leg.front {
            lame.part_health[leg.foot] = 0;
        }
    }
    route(
        "the shoulders, both forefeet broken",
        top(&lame, monster::SHOULDERS),
    );
    let mut best = Fx::MAX;
    for left in 0..t::topple_frames() {
        best = best.min(top(&holding(Doing::Toppled { left }), monster::BARREL));
    }
    route("the barrel, toppled", best);
    route(
        "the shoulders, through a slam",
        lowest_through(monster::SLAM, monster::SHOULDERS),
    );
    route(
        "the tail, through a sweep",
        lowest_through(monster::SWEEP, monster::TAIL_BASE),
    );

    let rig = standing.rig();
    let nose = rig
        .part_to_world(monster::HEAD, monster::shape(monster::HEAD).max)
        .x;
    let tip = rig
        .part_to_world(monster::TAIL_TIP, monster::shape(monster::TAIL_TIP).min)
        .x;
    println!(
        "\nnose to tail: {} m.  clips baked: {}",
        m(nose.sub(tip)),
        Clip::ALL.len()
    );
}
