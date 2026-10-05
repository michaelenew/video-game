//! **The movement envelope**: what each of the six classes can do with its
//! feet, played in the lab rather than read off the prose.
//!
//!     cargo run --release -p sim --bin envelope
//!     cargo run --release -p sim --bin envelope -- jump gaps flat search
//!     cargo run --release -p sim --bin envelope -- search --fixtures 2> crates/sim/tests/fixtures/envelope.txt
//!
//! The instrument the jump courses were built against; every number in
//! `docs/design/courses.md` §1 is printed here. See `sim::envelope` for how
//! each is measured. Release mode: the tool tables play a few thousand
//! attempts each.
//!
//! Integer arithmetic throughout: this file lives under `crates/sim/src`, and
//! the no-floats guard covers it.

use sim::arena::cm;
use sim::class::Class;

/// The five classes the courses are for. The Bulwark is out of this
/// exercise by the owner's choice (`docs/design/courses.md` §0).
const CLASSES: [Class; 5] = [
    Class::ShadowReaver,
    Class::Elementalist,
    Class::BloodMage,
    Class::DualMage,
    Class::Champion,
];
use sim::envelope::{self as e, Extra};
use sim::search as s;
use sim::{Fx, TICK_HZ};

fn main() {
    let asked: Vec<String> = std::env::args().skip(1).collect();
    let wanted = |name: &str| asked.is_empty() || asked.iter().any(|a| a == name);
    println!("The movement envelope, played in the lab (`--arena lab`).\n");
    if wanted("jump") {
        jump();
    }
    if wanted("gaps") {
        gaps();
    }
    if wanted("flat") {
        flat();
    }
    if wanted("search") {
        search();
    }
    if wanted("range") {
        range();
    }
}

fn secs(frames: u32) -> String {
    let tenths = frames * 10 / TICK_HZ;
    format!("{}.{}s", tenths / 10, tenths % 10)
}

fn jump() {
    println!("jump -- on the flat; long jumps off the runway's edge, landing level");
    println!(
        "  {:<14} {:>6} {:>12} {:>12} {:>9} {:>11}",
        "class", "run", "short hop", "full hop", "long jmp", "+airdodge"
    );
    for class in CLASSES {
        let (short, short_f) = e::hop(class, 1);
        let (full, full_f) = e::hop(class, 60);
        let plain = e::level_distance(&e::off_the_edge(class, Extra::Nothing));
        let dodged = e::airdodge_paths(class)
            .iter()
            .map(e::level_distance)
            .fold(Fx::ZERO, Fx::max);
        println!(
            "  {:<14} {:>4}/s {:>5}m {:>5} {:>5}m {:>5} {:>8}m {:>10}m",
            class.name(),
            e::metres(e::run_speed(class)),
            e::metres(short),
            secs(short_f),
            e::metres(full),
            secs(full_f),
            e::metres(plain),
            e::metres(dodged),
        );
    }
    println!();
}

/// The rises the gap table is read at, in centimetres.
const RISES: [i32; 9] = [-800, -400, -200, 0, 100, 200, 300, 400, 500];

fn gaps() {
    println!("gaps -- the widest gap (m) a running jump lands across, edge to edge, onto a");
    println!("ledge this far above (+) or below (-) the takeoff; plain / with the airdodge");
    print!("  {:<14}", "class");
    for r in RISES {
        print!(" {:>11}", format!("{}m", e::metres(cm(r))));
    }
    println!();
    for class in CLASSES {
        let plain = [e::off_the_edge(class, Extra::Nothing)];
        let dodged = e::airdodge_paths(class);
        print!("  {:<14}", class.name());
        for r in RISES {
            let a = e::widest(&plain, cm(r));
            let b = e::widest(&dodged, cm(r));
            print!(" {:>11}", format!("{}/{}", show(a), show(b)));
        }
        println!();
    }
    // The Dual mage with her second jump, at the tier that grants it.
    let second: Vec<_> = (19..64u32)
        .step_by(3)
        .map(|k| e::off_the_edge(Class::DualMage, Extra::SecondJump(k)))
        .collect();
    print!("  {:<14}", "Dual, 2nd jmp");
    for r in RISES {
        print!(" {:>11}", show(e::widest(&second, cm(r))));
    }
    println!("   (bars held at 80: set, not earned)");
    let blink: Vec<_> = (19..64u32)
        .step_by(6)
        .flat_map(|k| (2..40u32).step_by(6).map(move |b| (k, b)))
        .map(|(k, b)| e::off_the_edge(Class::DualMage, Extra::SecondJumpBlink(k, b)))
        .collect();
    print!("  {:<14}", "Dual, +blink");
    for r in RISES {
        print!(" {:>11}", show(e::widest(&blink, cm(r))));
    }
    println!("   (second jump, then the airborne blink)");
    println!("  '-' is nothing lands even at 0.1 m. Space is pressed on the last frame before");
    println!("  the edge; gap is from that edge to the ledge's face.\n");
}

fn show(v: Option<Fx>) -> String {
    v.map_or("-".to_string(), e::metres)
}

fn flat() {
    println!("flat -- the tools on open floor");
    let (range, dash_jump, apex) = e::reaver_chain();
    println!(
        "  Reaver        shadow sent level goes {} m; the dash jump out of the dash to it \
         carries {} m past it, {} m high",
        e::metres(range),
        e::metres(dash_jump),
        e::metres(apex)
    );
    // The dash jump, swept: every send distance and every frame of the carry.
    for strafe in [false] {
        let mut best: Option<(Fx, Fx, i32, u32)> = None;
        for send in [3, 5, 7, 9] {
            for k in 0..20u32 {
                if let Some((total, past)) = e::dash_jump(Fx::from_int(send), k, strafe) {
                    if best.is_none_or(|b| total.raw() > b.0.raw()) {
                        best = Some((total, past, send, k));
                    }
                }
            }
        }
        if let Some((total, past, send, k)) = best {
            let near = cm(50);
            let window = (0..20u32)
                .filter(|j| {
                    e::dash_jump(Fx::from_int(send), *j, strafe)
                        .is_some_and(|(t, _)| t.raw() >= total.sub(near).raw())
                })
                .count();
            println!(
                "                dash jump{}: best {} m from where she stood ({} past the shadow), \
                 sent {send} m, space {k} frames into the carry; {window} carry frames land within \
                 half a metre of it",
                if strafe {
                    ", strafing with an aerial every 20 frames"
                } else {
                    ", stick forward"
                },
                e::metres(total),
                e::metres(past)
            );
        }
    }
    let (single, double, draft, stone) = e::stone_apexes();
    println!(
        "  Elementalist  one-stone jump {} m, two-stone {} m, Updraft {} m, a stone stands {} m",
        e::metres(single),
        e::metres(double),
        e::metres(draft),
        e::metres(stone)
    );
    let tiers = e::shadowbox();
    let at = |t: Option<u32>| t.map_or("never".to_string(), secs);
    println!(
        "  Dual mage     shadowboxing (autos alternated, nothing hit) reaches blink {}, second \
         jump {}, wings {}",
        at(tiers[0]),
        at(tiers[1]),
        at(tiers[2])
    );
    let (ground, air) = e::dual_blink();
    println!(
        "                blink {} m standing, {} m airborne",
        e::metres(ground),
        e::metres(air)
    );
    for beat in [12u32, 20, 30] {
        let (high, far) = e::wings(beat);
        println!(
            "                ascending, space every {beat} frames: {} m up, {} m along before the \
             feet are down",
            e::metres(high),
            e::metres(far)
        );
    }
    println!("  Champion      one Rush: {} m", e::metres(e::rush()));
    let (high, far) = e::vault();
    println!(
        "                pole vault: {} m high, {} m along",
        e::metres(high),
        e::metres(far)
    );
    let (on, range) = e::blood_blink();
    println!(
        "  Blood mage    dodge-as-blink is {} ({} m if it were on); the pool blink needs a pool, \
         and pools come from cutting somebody -- not measured: a course has nobody to cut",
        if on { "on" } else { "off" },
        e::metres(range)
    );
    println!();
}

/// Runs per search: `--budget <n>`.
fn budget() -> usize {
    let args: Vec<String> = std::env::args().collect();
    args.iter()
        .position(|a| a == "--budget")
        .and_then(|i| args.get(i + 1))
        .and_then(|n| n.parse().ok())
        .unwrap_or(1500)
}

/// Restarts per search: `--restarts <n>`.
fn restarts() -> u64 {
    let args: Vec<String> = std::env::args().collect();
    args.iter()
        .position(|a| a == "--restarts")
        .and_then(|i| args.get(i + 1))
        .and_then(|n| n.parse().ok())
        .unwrap_or(3)
}

/// The best of several searches from different seeds: the search is a hill
/// climb, and one climb finds one hill.
fn best_of(
    stage: &s::Stage,
    kit: s::Kit,
    budget: usize,
    seed: u64,
    extra: Vec<s::Program>,
) -> (s::Program, s::Outcome) {
    let mut best: Option<(i64, s::Program, s::Outcome)> = None;
    for r in 0..restarts() {
        let (p, o) = s::search_from(stage, kit, budget, seed ^ (r << 40), extra.clone());
        let score = s::score(stage, &o);
        if best.as_ref().is_none_or(|b| score > b.0) {
            best = Some((score, p, o));
        }
    }
    let (_, p, o) = best.expect("at least one restart");
    (p, o)
}

/// **The search** (`sim::search`): for every class and every lane of the lab,
/// the widest gap off the runway onto a ledge at that rise -- with the shared
/// blocks (the jump, the airdodge, aerials, the strafe), and with the whole
/// kit -- and how many frames the tightest input of the best line can move.
/// `--fixtures` also prints the best programs to stderr, as
/// `tests/fixtures/envelope.txt` lines.
fn search() {
    let fixtures = std::env::args().any(|a| a == "--fixtures");
    let budget = budget();
    {
        println!("search -- the widest gap (m) found off the runway onto a ledge at each rise,");
        println!("shared blocks / whole kit, and [frames the tightest input can move, of 31]");
        println!(
            "(best of {} searches of {budget} runs per cell; a search is a lower bound)",
            restarts()
        );
        print!("  {:<9}", "rise");
        for (_, r) in sim::arena::lab::LANES {
            print!(" {:>13}", format!("{}m", e::metres(cm(r))));
        }
        println!();
    }
    for class in CLASSES {
        print!("  {:<9}", short(class));
        for (lane, (_, rise)) in sim::arena::lab::LANES.iter().enumerate() {
            let stage = s::Stage::lane(class, lane);
            let seed = (class as u64) << 16 | lane as u64;
            let (shared, so) = best_of(&stage, s::Kit::Shared, budget, seed, Vec::new());
            let (full, fo) = best_of(&stage, s::Kit::Full, budget, seed, vec![shared.clone()]);
            // The whole kit is at least the shared blocks.
            let (full, fo) = if so.landed && (!fo.landed || so.gap.raw() > fo.gap.raw()) {
                (shared.clone(), so)
            } else {
                (full, fo)
            };
            if fixtures {
                for (kit, p, o) in [("shared", &shared, so), ("full", &full, fo)] {
                    if o.landed {
                        eprintln!(
                            "{} {} {} {} | {}",
                            class.name().replace(' ', "_"),
                            lane,
                            kit,
                            sim::envelope::to_cm(o.gap),
                            p
                        );
                    }
                }
            }
            let (win, _) = s::window(&stage, &full, cm(50));
            let cell = |o: &s::Outcome| {
                if o.landed {
                    e::metres(o.gap)
                } else {
                    "-".to_string()
                }
            };
            print!(
                " {:>13}",
                format!(
                    "{}/{} [{}]",
                    cell(&so),
                    cell(&fo),
                    if fo.landed { win } else { 0 }
                )
            );
            let _ = rise;
        }
        println!();
    }
    println!();
}

/// **The Reaver at a shorter shadow range** -- the owner's leading idea for
/// making her shadow less trivial. The send's reach set in the Oven for the
/// length of the run, and the whole-kit search on a few lanes: how far she
/// gets, against what the shared blocks alone get her at that rise.
fn range() {
    use sim::oven::{MoveField, set_move_field};
    let budget = budget();
    let lanes = [1usize, 3, 5, 7, 8];
    println!("range -- the Reaver's widest gap (m), whole kit, with the shadow's send reach");
    println!("set to each of these ({budget} runs per cell); 'shared' is no shadow at all");
    print!("  {:<9}", "reach");
    for l in lanes {
        print!(
            " {:>8}",
            format!("{}m", e::metres(cm(sim::arena::lab::LANES[l].1)))
        );
    }
    println!();
    let slot = sim::state::SLOT_MECHANIC as usize;
    let before = sim::oven::move_field(Class::ShadowReaver, slot, MoveField::Reach);
    print!("  {:<9}", "shared");
    let mut shared = Vec::new();
    for l in lanes {
        let st = s::Stage::lane(Class::ShadowReaver, l);
        let (p, o) = best_of(&st, s::Kit::Shared, budget, l as u64, Vec::new());
        print!(
            " {:>8}",
            if o.landed {
                e::metres(o.gap)
            } else {
                "-".to_string()
            }
        );
        shared.push(p);
    }
    println!();
    for reach in [3, 5, 7, 9] {
        set_move_field(
            Class::ShadowReaver,
            slot,
            MoveField::Reach,
            Fx::from_int(reach).raw(),
        );
        print!("  {:<9}", format!("{reach} m"));
        for (i, l) in lanes.iter().enumerate() {
            let st = s::Stage::lane(Class::ShadowReaver, *l);
            let (_, o) = best_of(
                &st,
                s::Kit::Full,
                budget,
                *l as u64,
                vec![shared[i].clone()],
            );
            print!(
                " {:>8}",
                if o.landed {
                    e::metres(o.gap)
                } else {
                    "-".to_string()
                }
            );
        }
        println!();
    }
    set_move_field(Class::ShadowReaver, slot, MoveField::Reach, before);
    println!();
}

fn short(c: Class) -> &'static str {
    match c {
        Class::Bulwark => "Bulwark",
        Class::Champion => "Champion",
        Class::ShadowReaver => "Reaver",
        Class::Elementalist => "Elementalist",
        Class::BloodMage => "Blood mage",
        Class::DualMage => "Dual mage",
    }
}
