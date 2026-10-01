//! What the creature actually measures, in the states that matter.
//!
//!     cargo run -p sim --bin beastcheck
//!     cargo run -p sim --bin beastcheck -- --species ridgeback
//!
//! Any registered species; the Ridgeback by default. Everything it prints is
//! read off the species' table -- which parts are mountable, which are weak,
//! which legs it has, which moves -- so a new creature is measured the moment
//! it is registered.
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

use sim::fixed::Fx;
use sim::math::V3;
use sim::monster::{Doing, Monster};
use sim::oven::Unit;
use sim::species::Species;
use sim::state::MAX_PLAYERS;

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
    let sh = beast.sp().shape(part);
    let mid = |a: Fx, b: Fx| a.add(b).mul(Fx::ratio(1, 2));
    beast
        .rig()
        .part_to_world(
            part,
            V3::new(mid(sh.min.x, sh.max.x), sh.max.y, mid(sh.min.z, sh.max.z)),
        )
        .y
}

fn holding(sp: &Species, doing: Doing) -> Monster {
    let mut beast = Monster::new(sp.id);
    beast.doing = doing;
    beast
}

/// The world height of a part's lowest corner.
fn bottom(beast: &Monster, part: usize) -> Fx {
    let sh = beast.sp().shape(part);
    let rig = beast.rig();
    (0..8)
        .map(|c| {
            let p = V3::new(
                if c & 1 == 0 { sh.min.x } else { sh.max.x },
                if c & 2 == 0 { sh.min.y } else { sh.max.y },
                if c & 4 == 0 { sh.min.z } else { sh.max.z },
            );
            rig.part_to_world(part, p).y
        })
        .min_by_key(|y| y.raw())
        .unwrap_or(Fx::ZERO)
}

/// Does a part's top face point up enough to stand on? A creature on its back
/// has every "top" facing the floor, and one rolling over has them on edge: a
/// face steeper than forty-five degrees is a wall.
fn faces_up(beast: &Monster, part: usize) -> bool {
    let up = beast
        .rig()
        .of(part)
        .rot
        .apply(V3::new(Fx::ZERO, Fx::ONE, Fx::ZERO));
    // Steeper than forty-five degrees: more across than up.
    up.y.raw() > V3::new(up.x, Fx::ZERO, up.z).flat_len().raw()
}

/// The lowest mountable surface on a body, and which part it is.
fn lowest_mount(beast: &Monster) -> Option<(usize, Fx)> {
    let sp = beast.sp();
    (0..sp.parts.len())
        .filter(|p| sp.parts[*p].shape.mountable && faces_up(beast, *p))
        .map(|p| (p, top(beast, p)))
        .min_by_key(|(_, h)| h.raw())
}

/// The lowest any mountable surface gets across a run of states.
fn lowest_across(states: impl Iterator<Item = Monster>) -> Option<(usize, Fx)> {
    states
        .filter_map(|b| lowest_mount(&b))
        .min_by_key(|(_, h)| h.raw())
}

/// Every frame of a move, as the body holds it.
fn through(sp: &Species, kind: u8) -> impl Iterator<Item = Monster> + '_ {
    let a = sp.attack(kind);
    [(0u8, a.startup), (1, a.active), (2, a.recovery)]
        .into_iter()
        .flat_map(move |(which, span)| {
            (0..span).map(move |left| {
                holding(
                    sp,
                    match which {
                        0 => Doing::Startup { kind, left },
                        1 => Doing::Active { kind, left },
                        _ => Doing::Recovery { kind, left },
                    },
                )
            })
        })
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let wanted = args
        .iter()
        .position(|a| a == "--species")
        .and_then(|i| args.get(i + 1))
        .map_or("ridgeback", |s| s.as_str());
    let Some(sp) = sim::species::named(wanted) else {
        let known: Vec<&str> = sim::species::all().map(|s| s.name).collect();
        eprintln!("no species called {wanted}; there is {}", known.join(", "));
        std::process::exit(2);
    };
    if !sp.has_body() && sp.pack.is_some_and(|p| p.kinds.iter().any(|k| k.mountable)) {
        // A pack with a back to ride (the Hornback's cow): how high it is,
        // against how high every class hops, and whether each one gets on.
        println!("the {}: backs to ride\n", sp.name);
        let decl = sp.pack.unwrap();
        for (k, kind) in decl.kinds.iter().enumerate() {
            if !kind.mountable {
                continue;
            }
            let tall = sim::critter::stat_fx(sp, k as u8, sim::critter::CritterField::Height);
            println!("  {:<10} back at {} m", kind.name, m(tall));
            let mut all = true;
            for class in sim::class::ALL_CLASSES {
                let apex = jump_apex(class);
                let on = sim::critcheck::lands_on(class, sp.id, k as u8);
                all &= on.is_some();
                println!(
                    "    {:<13} hop {} m, {} m to spare: {}",
                    class.name(),
                    m(apex),
                    m(apex.sub(tall)),
                    if on.is_some() { "gets on" } else { "does not" }
                );
            }
            println!(
                "  every class can jump onto a {}: {}\n",
                kind.name.to_lowercase(),
                if all { "yes" } else { "no" }
            );
        }
        return;
    }
    if !sp.has_body() {
        eprintln!(
            "the {} have no body to climb: they are a pack. `cargo run -p sim --bin critcheck -- --species {}` measures small bodies.",
            sp.name,
            sp.slug()
        );
        std::process::exit(2);
    }
    println!("the {}\n", sp.name);

    // **Two jumps, not one.** This printed a single apex until 2026-09-17,
    // which was honest while the cast spanned four metres to seven and a half:
    // the shortest hop in the game got you most of the way up, so answering for
    // the heaviest class answered for everybody. The mobility pass took the
    // whole cast down and left a spread of better than two to one, so a single
    // number now hides the thing the climb turns on -- which classes can get up
    // there at all.
    let mut lowest = (sim::Class::Bulwark, Fx::MAX);
    let mut highest = (sim::Class::Bulwark, Fx::ZERO);
    let mut apexes: Vec<(sim::Class, Fx)> = Vec::new();
    for class in sim::class::ALL_CLASSES {
        let apex = jump_apex(class);
        apexes.push((class, apex));
        if apex.raw() < lowest.1.raw() {
            lowest = (class, apex);
        }
        if apex.raw() > highest.1.raw() {
            highest = (class, apex);
        }
    }
    let (short, apex) = lowest;
    let (tall, best) = highest;
    let platform = sim::arena::proving_ground::wall_height();
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

    // **Who can get there from the floor, by name.** "The floatier classes"
    // was the label for anything between the shortest hop and the tallest,
    // and once the animal grew a metre that band held every surface on it
    // while the honest answer for most of them was "the Dual mage". A route
    // is a route for the classes that can jump it, and the climb turns on
    // which those are.
    let reach = |h: Fx| -> String {
        let can: Vec<&str> = apexes
            .iter()
            .filter(|(_, a)| h.raw() <= a.raw())
            .map(|(c, _)| c.name())
            .collect();
        if can.len() == apexes.len() {
            "a standing jump, every class".to_string()
        } else if !can.is_empty() {
            format!("a standing jump for {}", can.join(", "))
        } else if h.raw() <= from_platform.raw() {
            "only from a platform".to_string()
        } else {
            "out of reach".to_string()
        }
    };

    let standing = holding(sp, Doing::Prowl);
    println!("standing, the tops of the surfaces you can stand on:");
    for (part, p) in sp.parts.iter().enumerate() {
        if !p.shape.mountable {
            continue;
        }
        let h = top(&standing, part);
        println!("  {:<14} {:>6} m   {}", p.name, m(h), reach(h));
    }

    // **The breakables against the hops**: each one's lowest corner standing,
    // and the lowest it comes in any move -- the Broodmother's sacs are not
    // weak points but the fight turns on which hops reach them (her §1), so
    // they are measured here rather than argued about.
    let breakable: Vec<usize> = (0..sp.parts.len())
        .filter(|p| sp.parts[*p].shape.breakable && !sp.parts[*p].weak)
        .collect();
    if !breakable.is_empty() {
        println!(
            "\nthe breakable parts, their lowest corner (a hop's feet, before the swing's reach):"
        );
        for part in breakable {
            let low = bottom(&standing, part);
            let (kind, lowest) = (0..sp.moves.len() as u8)
                .map(|k| {
                    let h = through(sp, k)
                        .map(|b| bottom(&b, part))
                        .min_by_key(|h| h.raw())
                        .unwrap_or(low);
                    (k, h)
                })
                .min_by_key(|(_, h)| h.raw())
                .unwrap_or((0, low));
            println!(
                "  {:<16} {:>6} m standing, {}; {:>6} m in the {}, {}",
                sp.parts[part].name,
                m(low),
                reach(low),
                m(lowest),
                sp.moves[kind as usize].name.to_lowercase(),
                reach(lowest)
            );
        }
    }

    println!("\nthe weak points, standing:");
    for (part, p) in sp.parts.iter().enumerate() {
        if !p.weak {
            continue;
        }
        let sh = sp.shape(part);
        let high = top(&standing, part);
        println!(
            "  {:<14} {:>6} m at its foot, {:>6} m at its top, x{} damage",
            p.name,
            m(high.sub(sh.max.y.sub(sh.min.y))),
            m(high),
            m(sp.vulnerability(part))
        );
    }

    // **What opens a way up**: in every state that lowers the body, the
    // lowest surface it offers and which one that is. Read off the table --
    // its stock states, its broken legs, every move it has -- so nothing
    // about a particular animal is written here.
    println!("\nand what opens a way up (the lowest surface, at its lowest):");
    let route = |label: &str, found: Option<(usize, Fx)>| {
        if let Some((part, h)) = found {
            println!(
                "  {label:<30} {:<14} {:>6} m   {}",
                sp.parts[part].name,
                m(h),
                reach(h)
            );
        }
    };
    // One stumble: which end went down is in the snapshot, but the clip is
    // the same either way.
    route(
        "stumbling",
        lowest_across(
            (0..sp.stumble_frames()).map(|left| holding(sp, Doing::Stumble { left, front: true })),
        ),
    );
    route(
        "toppled",
        lowest_across((0..sp.topple_frames()).map(|left| holding(sp, Doing::Toppled { left }))),
    );
    for (label, front) in [
        ("every front foot broken", true),
        ("every hind foot broken", false),
    ] {
        let mut lame = Monster::new(sp.id);
        let mut any = false;
        for leg in sp.legs.iter().filter(|l| l.front == front) {
            if let Some(slot) = sp.break_slot(leg.foot) {
                lame.breaks[slot] = 0;
                any = true;
            }
        }
        if any {
            route(label, lowest_mount(&lame));
        }
    }
    for (kind, decl) in sp.moves.iter().enumerate() {
        route(
            &format!("through {}", decl.name.to_lowercase()),
            lowest_across(through(sp, kind as u8)),
        );
    }

    // Length along the body, from the frontmost point of any part to the
    // rearmost, standing.
    let rig = standing.rig();
    let mut front = Fx::MIN;
    let mut back = Fx::MAX;
    for part in 0..sp.parts.len() {
        let sh = sp.shape(part);
        for x in [sh.min.x, sh.max.x] {
            for z in [sh.min.z, sh.max.z] {
                for y in [sh.min.y, sh.max.y] {
                    let at = rig.to_body(rig.part_to_world(part, V3::new(x, y, z))).x;
                    front = front.max(at);
                    back = back.min(at);
                }
            }
        }
    }
    println!(
        "\nnose to tail: {} m.  clips baked: {}",
        m(front.sub(back)),
        sp.clips.len()
    );
}
