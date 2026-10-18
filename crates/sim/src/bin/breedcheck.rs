//! Print what the creature builder makes of the cast, and watch a few
//! lineages drift.
//!
//! ```text
//! cargo run -p sim --bin breedcheck            # the cast, rebuilt
//! cargo run -p sim --bin breedcheck -- feathers [seed]   # a Ridgeback given feathers
//! cargo run -p sim --bin breedcheck -- mantis [seed]     # a Ridgeback given a Mantis's claw
//! cargo run -p sim --bin breedcheck -- drift [seed]      # every creature, forty generations alone
//! cargo run -p sim --bin breedcheck -- herd [seed]       # breeding the biggest Ridgebacks
//! cargo run -p sim --bin breedcheck -- boxes <egg> <donor|none> <trophy|none> <every> <seed> <gens> [phase [fly]]
//!                                            # every generation's body as side-view boxes, for a sheet
//! ```

use sim::breed::archetypes::{self, CAST, GALEWING, MANTIS, RIDGEBACK};
use sim::breed::build;
use sim::breed::cross::{self, Rates, Trophy, TrophyKind};
use sim::breed::plan;
use sim::breed::{Genome, Rng, Role};
use sim::fixed::Fx;

/// A fixed-point number for printing, with `{:.N}` honoured by integer
/// arithmetic: this file is under `crates/sim/src`, where the no-floats
/// guard holds, and a diagnostic that rounds through `f32` reports on a
/// different number from the one the simulation runs.
#[derive(Clone, Copy)]
struct D(Fx);

impl std::fmt::Display for D {
    fn fmt(&self, out: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let places = out.precision().unwrap_or(2).min(4) as u32;
        let scale = 10i64.pow(places);
        let raw = self.0.raw() as i64;
        // Round half away from zero, in 16.16.
        let scaled = (raw.abs() * scale + (1 << 15)) >> 16;
        let sign = if raw < 0 && scaled != 0 { "-" } else { "" };
        if places == 0 {
            return write!(out, "{sign}{scaled}");
        }
        write!(
            out,
            "{sign}{}.{:0width$}",
            scaled / scale,
            scaled % scale,
            width = places as usize
        )
    }
}

fn f(x: Fx) -> D {
    D(x)
}

/// A decimal argument ("0.3") as fixed point, read without a float.
fn parse_fx(s: &str) -> Option<Fx> {
    let (whole, frac) = s.split_once('.').unwrap_or((s, ""));
    let whole: i32 = if whole.is_empty() {
        0
    } else {
        whole.parse().ok()?
    };
    let digits: String = frac.chars().take(4).collect();
    let den = 10i32.pow(digits.len() as u32);
    let num: i32 = if digits.is_empty() {
        0
    } else {
        digits.parse().ok()?
    };
    Some(Fx::from_int(whole).add(Fx::ratio(num, den)))
}

fn describe(g: &Genome) -> String {
    let limbs: Vec<String> = g
        .pairs()
        .map(|(s, p)| {
            format!(
                "{:?}:{:?}/{:?}/{:?} {:.2}x{:.2}",
                s,
                p.role,
                p.build,
                p.tip,
                f(p.length),
                f(p.girth)
            )
        })
        .collect();
    let s = &g.sliders;
    format!(
        "{:?} {:?} thorax={} abdomen={} tail={:?} | {} | {:?} {:?} {:?} {:?} {:?} | size {:.2} mass {:.2} pitch {:.2} speed {:.2} react {:.2} aggr {:.2} tempo {:.2} guard {:.2}",
        g.stance,
        g.head,
        g.segments.thorax,
        g.segments.abdomen,
        g.segments.tail,
        limbs.join(", "),
        g.minors.jaw,
        g.minors.crown,
        g.minors.tail_tip,
        g.minors.hide,
        g.minors.back,
        f(s.size),
        f(s.mass),
        f(s.pitch),
        f(s.speed),
        f(s.reaction),
        f(s.aggression),
        f(s.tempo),
        f(s.guard)
    )
}

fn verdict(g: &Genome) -> String {
    let a = plan::assess(g);
    let mut caps = Vec::new();
    if a.caps.flies {
        caps.push("flies");
    }
    if a.caps.vestigial_wings {
        caps.push("vestigial wings");
    }
    if a.caps.charges {
        caps.push("charges");
    }
    if a.caps.pounces {
        caps.push("pounces");
    }
    if a.caps.guards {
        caps.push("guards");
    }
    if a.caps.burrows {
        caps.push("burrows");
    }
    if a.caps.mountable {
        caps.push("mountable");
    }
    if a.caps.swallows {
        caps.push("swallows");
    }
    format!(
        "tier {} plausibility {:.2} [{}]",
        a.tier,
        f(a.plausibility),
        caps.join(", ")
    )
}

fn cast() {
    for (id, name, g, kinds) in CAST {
        let body = build::body(g);
        let moves = build::moves(g);
        let real = id.get();
        println!("== {name}");
        println!("   {}", describe(g));
        println!("   {}", verdict(g));
        if real.has_body() {
            println!(
                "   body: {} legs (table {}), hips {:.2} m (table {:.2}), length {:.1} m, top {:.1} m, span {:.1} m, {} bones, {} parts",
                body.leg_count,
                real.legs.len(),
                f(body.hips),
                f(real.bones[0].rest.y),
                f(body.length),
                f(body.top()),
                f(body.span),
                body.bone_count,
                body.part_count
            );
        } else {
            println!(
                "   body: {} legs, hips {:.2} m, length {:.1} m, top {:.1} m (a pack: no table body)",
                body.leg_count,
                f(body.hips),
                f(body.length),
                f(body.top())
            );
        }
        let names: Vec<String> = moves
            .iter()
            .map(|m| {
                let wanted = kinds.contains(&m.kind);
                format!(
                    "{}{} ({}/{}/{} {}dmg {:.1}m w{:.2})",
                    if wanted { "*" } else { "" },
                    m.kind.name(),
                    m.startup,
                    m.active,
                    m.recovery,
                    m.damage,
                    f(m.reach),
                    f(m.weight)
                )
            })
            .collect();
        println!("   moves: {}", names.join(", "));
        let missing: Vec<&str> = kinds
            .iter()
            .filter(|k| !moves.has(**k))
            .map(|k| k.name())
            .collect();
        if !missing.is_empty() {
            println!("   MISSING from the table's kinds: {}", missing.join(", "));
        }
        let drops: Vec<&str> = archetypes::drops(g)
            .iter()
            .flatten()
            .map(|t| t.name())
            .collect();
        println!("   drops: {}", drops.join(", "));
    }
}

fn lineage(name: &str, egg: Genome, trophy: Trophy, gens: u32, seed: u64) {
    println!(
        "== {name}: seed {seed}, {gens} generations of {}",
        trophy.kind.name()
    );
    let mut rng = Rng::new(seed);
    let mut g = egg;
    let mut last = String::new();
    for generation in 1..=gens {
        let out = cross::graft(&g, &trophy, &Rates::DEFAULT, &mut rng);
        g = out.genome;

        let wing = g
            .pairs()
            .find(|(_, p)| p.role == Role::Wing)
            .map(|(_, p)| format!(" wing {:.2}", f(p.length)))
            .unwrap_or_default();
        let line = format!(
            "{:?} fore={} pitch {:.2} mass {:.2} size {:.2}{} margin {:.2} {}{}",
            g.stance,
            g.limbs[0]
                .map(|p| format!("{:?}/{:?}", p.role, p.build))
                .unwrap_or("none".into()),
            f(g.sliders.pitch),
            f(g.sliders.mass),
            f(g.sliders.size),
            wing,
            f(plan::flight_margin(&g)),
            verdict(&g),
            if out.fixes.count > 0 {
                format!(" fixes {:?}", out.fixes.iter().collect::<Vec<_>>())
            } else {
                String::new()
            }
        );
        if line != last || out.moved.any() {
            println!(
                "   generation {generation:3}: {line}{}",
                if out.moved.any() {
                    "  <- a major moved"
                } else {
                    ""
                }
            );
            last = line;
        }
    }
    println!("   final: {}", describe(&g));
    let moves: Vec<&str> = build::moves(&g).iter().map(|m| m.kind.name()).collect();
    println!("   moves: {}", moves.join(", "));
}

fn drift(seed: u64) {
    for (_, name, g, _) in CAST {
        let mut rng = Rng::new(seed);
        let mut x = *g;
        let mut majors = 0;
        let mut worst = Fx::ONE;
        for _ in 0..40 {
            let out = cross::mutate(&x, &Rates::DEFAULT, &mut rng);
            x = out.genome;
            if out.moved.any() {
                majors += 1;
            }
            worst = worst.min(plan::assess(&x).plausibility);
        }
        println!(
            "== {name}: {majors} major changes in 40 generations, worst plausibility {:.2}",
            f(worst)
        );
        println!("   {}", describe(&x));
    }
}

fn herd(seed: u64) {
    let mut rng = Rng::new(seed);
    let mut herd: Vec<Genome> = (0..8)
        .map(|i| archetypes::vary(&RIDGEBACK, seed * 8 + i))
        .collect();
    for generation in 0..20 {
        herd.sort_by_key(|g| std::cmp::Reverse(g.sliders.size));
        let sizes: Vec<String> = herd
            .iter()
            .map(|g| format!("{:.3}", f(g.sliders.size)))
            .collect();
        println!(
            "generation {generation:2}: sizes {} -> {:.1} m",
            sizes.join(" "),
            f(build::length_m(herd[0].sliders.size))
        );
        let (a, b) = (herd[0], herd[1]);
        herd = (0..8)
            .map(|_| cross::breed(&a, &b, &Rates::DEFAULT, &mut rng).genome)
            .collect();
    }
}

/// A creature by name, for the sheet's arguments.
fn by_name(name: &str) -> Option<&'static Genome> {
    CAST.iter()
        .find(|(id, n, _, _)| {
            n.eq_ignore_ascii_case(name) || id.get().name.eq_ignore_ascii_case(name)
        })
        .map(|(_, _, g, _)| *g)
}

/// Every generation of a lineage as its body's boxes in side view, one
/// line per part, for a sheet drawn outside the simulation.
///
/// ```text
/// gen <n> <stance> <plausibility> <tier> <caps...> | <genome>
/// obox <tag> <name> <cx> <cy> <cz> <ux> <uy> <uz> <along> <across> <up>
/// ```
///
/// A trophy of `kind` from `donor` is given every `every` generations;
/// `none` or `every` of zero is drift alone.
fn boxes(args: &[String]) {
    let egg = args
        .first()
        .and_then(|n| by_name(n))
        .copied()
        .unwrap_or(RIDGEBACK);
    let donor = args.get(1).and_then(|n| by_name(n)).copied();
    let kind = args
        .get(2)
        .and_then(|k| {
            TrophyKind::ALL
                .iter()
                .find(|t| t.name().replace(' ', "_") == *k)
        })
        .copied();
    let every: u32 = args.get(3).and_then(|s| s.parse().ok()).unwrap_or(0);
    let seed: u64 = args.get(4).and_then(|s| s.parse().ok()).unwrap_or(1);
    let gens: u32 = args.get(5).and_then(|s| s.parse().ok()).unwrap_or(40);
    // A seventh argument poses every body at that point of its stride.
    let phase: Option<Fx> = args.get(6).and_then(|s| parse_fx(s));
    // An eighth, `fly`, poses it in flight instead (`build::body_flying`).
    let flight = args.get(7).is_some_and(|a| a == "fly");
    let mut rng = Rng::new(seed);
    let mut g = egg;
    for generation in 0..=gens {
        if generation > 0 {
            let graft = every > 0 && generation % every == 0;
            g = match (graft, donor, kind) {
                (true, Some(donor), Some(kind)) => {
                    cross::graft(&g, &Trophy { kind, donor }, &Rates::DEFAULT, &mut rng).genome
                }
                _ => cross::mutate(&g, &Rates::DEFAULT, &mut rng).genome,
            };
        }
        let a = plan::assess(&g);
        println!(
            "gen {generation} {:?} {:.2} {} {} | {}",
            g.stance,
            f(a.plausibility),
            a.tier,
            verdict(&g)
                .split('[')
                .nth(1)
                .unwrap_or("")
                .trim_end_matches(']'),
            describe(&g)
        );
        let q = sim::breed::posture::posture(&g);
        let body = match phase {
            Some(ph) if flight => build::body_flying(&g, &q, ph),
            Some(ph) => build::body_posed(&g, &q, ph),
            None => build::body(&g),
        };
        // Every part as an oriented box: `obox <tag> <name> <cx> <cy> <cz>
        // <ux> <uy> <uz> <along> <across> <up>` -- its centre, the unit
        // axis its box runs along, and its half-extents in the axis's
        // frame (`build::frame`). The sheet draws these as the game would
        // draw the part in its bone's frame.
        for (i, part) in body.parts().iter().enumerate() {
            let name = part.name;
            let tag = if name.starts_with("wing_feather") {
                "feathered"
            } else if name.starts_with("wing_membrane") {
                "membrane"
            } else if name.starts_with("wing_fur") {
                "furred"
            } else if name.contains("foot") {
                "foot"
            } else if name.starts_with("fore")
                || name.starts_with("hind")
                || name.starts_with("mid")
            {
                "limb"
            } else if name.starts_with("head")
                || name == "jaw"
                || name == "horns"
                || name == "crest"
                || name == "neck"
            {
                "head"
            } else if name.starts_with("tail") {
                "tail"
            } else {
                "torso"
            };
            let c = body.centre(i);
            let u = body.axes[i];
            let h = part.shape.max.sub(part.shape.min).scale(Fx::ratio(1, 2));
            println!(
                "obox {tag} {} {:.3} {:.3} {:.3} {:.3} {:.3} {:.3} {:.3} {:.3} {:.3}",
                name.replace(' ', "_"),
                f(c.x),
                f(c.y),
                f(c.z),
                f(u.x),
                f(u.y),
                f(u.z),
                f(h.x),
                f(h.y),
                f(h.z)
            );
        }
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let seed = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(1u64);
    match args.first().map(String::as_str) {
        None | Some("cast") => cast(),
        Some("feathers") => lineage(
            "A Ridgeback given a Galewing's feathers",
            RIDGEBACK,
            Trophy {
                kind: TrophyKind::Feather,
                donor: GALEWING,
            },
            80,
            seed,
        ),
        Some("mantis") => lineage(
            "A Ridgeback given a Mantis's claw",
            RIDGEBACK,
            Trophy {
                kind: TrophyKind::Claw,
                donor: MANTIS,
            },
            40,
            seed,
        ),
        Some("drift") => drift(seed),
        Some("herd") => herd(seed),
        Some("boxes") => boxes(&args[1..]),
        Some(other) => println!("unknown: {other}"),
    }
}
