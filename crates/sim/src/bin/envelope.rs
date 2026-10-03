//! **The movement envelope**: what each of the six classes can do with its
//! feet, played in the lab rather than read off the prose.
//!
//!     cargo run --release -p sim --bin envelope
//!     cargo run --release -p sim --bin envelope -- jump gaps tools flat
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
    if wanted("tools") {
        tools();
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

fn tools() {
    println!("tools -- the furthest back (m) from a ledge's face she can stand, still, and end");
    println!("on its top; '-' is not from anywhere");
    print!("  {:<30}", "tool");
    for i in 0..e::ledges() {
        print!(" {:>6}", format!("{}m", e::metres(e::ledge(i).height)));
    }
    println!();
    for tool in e::tools() {
        let row = e::reach_table(&tool);
        print!("  {:<30}", format!("{} {}", short(tool.class), tool.name));
        for r in row {
            // The scan stops at 30 m: anything that reaches it reaches further.
            let shown = match r {
                Some(v) if v.raw() >= Fx::from_int(e::scan_metres()).raw() => "30+".to_string(),
                _ => show(r),
            };
            print!(" {:>6}", shown);
        }
        println!();
    }
    println!();
    for tool in e::tools() {
        println!("  {} {}: {}", short(tool.class), tool.name, tool.note);
    }
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
