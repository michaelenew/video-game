//! Print what the creature builder makes of the cast, and watch a few
//! lineages drift.
//!
//! ```text
//! cargo run -p sim --bin breedcheck            # the cast, rebuilt
//! cargo run -p sim --bin breedcheck -- feathers [seed]   # a Ridgeback given feathers
//! cargo run -p sim --bin breedcheck -- mantis [seed]     # a Ridgeback given a Mantis's claw
//! cargo run -p sim --bin breedcheck -- drift [seed]      # every creature, forty generations alone
//! cargo run -p sim --bin breedcheck -- herd [seed]       # breeding the biggest Ridgebacks
//! cargo run -p sim --bin breedcheck -- boxes <egg> <donor|none> <trophy|none> <every> <seed> <gens>
//!                                            # every generation's body as side-view boxes, for a sheet
//! ```

use sim::breed::archetypes::{self, CAST, GALEWING, MANTIS, RIDGEBACK};
use sim::breed::build;
use sim::breed::cross::{self, Rates, Trophy, TrophyKind};
use sim::breed::plan;
use sim::breed::{Genome, Rng, Role};
use sim::fixed::Fx;

fn f(x: Fx) -> f32 {
    x.to_f32_for_render()
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
        let mut worst = 1.0f32;
        for _ in 0..40 {
            let out = cross::mutate(&x, &Rates::DEFAULT, &mut rng);
            x = out.genome;
            if out.moved.any() {
                majors += 1;
            }
            worst = worst.min(f(plan::assess(&x).plausibility));
        }
        println!(
            "== {name}: {majors} major changes in 40 generations, worst plausibility {worst:.2}"
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
        herd.sort_by(|a, b| b.sliders.size.cmp(&a.sliders.size));
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
/// part <tag> <name> <x0> <y0> <x1> <y1> <z0> <z1>
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
        let body = build::body(&g);
        for part in body.parts() {
            let at = body.at(part.shape.bone);
            let name = part.name;
            let tag = if name.contains("foot") {
                "foot"
            } else if name.starts_with("fore")
                || name.starts_with("hind")
                || name.starts_with("mid")
            {
                let wing = g.pairs().any(|(st, p)| {
                    p.role == Role::Wing
                        && name.starts_with(match st {
                            sim::breed::Station::Fore => "fore",
                            sim::breed::Station::MidFore => "midfore",
                            sim::breed::Station::MidHind => "midhind",
                            sim::breed::Station::Hind => "hind",
                        })
                });
                if wing { "wing" } else { "limb" }
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
            println!(
                "part {tag} {} {:.3} {:.3} {:.3} {:.3} {:.3} {:.3}",
                name.replace(' ', "_"),
                f(at.x.add(part.shape.min.x)),
                f(at.y.add(part.shape.min.y)),
                f(at.x.add(part.shape.max.x)),
                f(at.y.add(part.shape.max.y)),
                f(at.z.add(part.shape.min.z)),
                f(at.z.add(part.shape.max.z))
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
