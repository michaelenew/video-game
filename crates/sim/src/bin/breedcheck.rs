//! Print what the creature builder makes of the cast, and watch a few
//! lineages drift.
//!
//! ```text
//! cargo run -p sim --bin breedcheck            # the cast, rebuilt
//! cargo run -p sim --bin breedcheck -- feathers [seed]   # a Ridgeback given feathers
//! cargo run -p sim --bin breedcheck -- mantis [seed]     # a Ridgeback given a Mantis's claw
//! cargo run -p sim --bin breedcheck -- drift [seed]      # every creature, forty generations alone
//! cargo run -p sim --bin breedcheck -- herd [seed]       # breeding the biggest Ridgebacks
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
        Some(other) => println!("unknown: {other}"),
    }
}
