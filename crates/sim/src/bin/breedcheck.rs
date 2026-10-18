//! Print what the creature builder makes of the cast, and watch a few
//! lineages drift.
//!
//! ```text
//! cargo run -p sim --bin breedcheck            # the cast, rebuilt
//! cargo run -p sim --bin breedcheck -- feathers [seed]   # a Ridgeback given feathers
//! cargo run -p sim --bin breedcheck -- mantis [seed]     # a Ridgeback given a Mantis's claw
//! cargo run -p sim --bin breedcheck -- drift [seed]      # every creature, forty generations alone
//! cargo run -p sim --bin breedcheck -- herd [seed]       # breeding the biggest Ridgebacks
//! cargo run -p sim --bin breedcheck -- moves [name]      # every move measured against the contract
//! cargo run -p sim --bin breedcheck -- film <name> [move|all] # every move's frames and hits, for a sheet
//! cargo run -p sim --bin breedcheck -- strides <name>     # every gait's cycle and footfalls, for a sheet
//! cargo run -p sim --bin breedcheck -- gaits             # every creature's gaits by speed
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

/// Every part of a body as an oriented box, one line each.
fn print_boxes(body: &build::Body) {
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
        } else if name.starts_with("fore") || name.starts_with("hind") || name.starts_with("mid") {
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

/// Every move of a creature (or the one named) as a strip of frames, for
/// `scripts/breed-motion.py`: rest, a third into the windup, the cocked
/// pose, the strike's middle and end, the end of the stuck window, and
/// recovered. Each frame's hit spheres are printed with it.
///
/// ```text
/// move <kind> <startup> <active> <recovery> <answer> <lane> <range> <beyond> <tell-ok> | <name>
/// frame <n> <beat>
/// obox ...
/// hit <x> <y> <z> <radius>
/// ```
fn film(args: &[String]) {
    use sim::breed::{posture, strike, verdict};
    let g = args
        .first()
        .and_then(|n| by_name(n))
        .copied()
        .unwrap_or(RIDGEBACK);
    let only = args.get(1).map(|s| s.replace('_', " "));
    let q = posture::posture(&g);
    let rest = build::body_in(&g, &q);
    let len = build::proportions(&g).len;
    let (kit, n) = strike::kit(&g, &q);
    for (spec, m) in kit[..n].iter().flatten() {
        if only
            .as_deref()
            .is_some_and(|o| o != "all" && o != spec.kind.name())
        {
            continue;
        }
        let r = verdict::read(&g, &q, &rest, spec, m);
        println!(
            "move {} {} {} {} {} {} {} {:.1} {} | {}",
            spec.kind.name().replace(' ', "_"),
            spec.startup,
            spec.active,
            spec.recovery,
            r.answer.name().replace(' ', "_"),
            r.lane.name(),
            r.range.name(),
            f(r.beyond),
            if r.unanswerable(len) {
                "UNANSWERABLE"
            } else if r.damaging {
                "ok"
            } else {
                "no-hit"
            },
            spec.kind.name()
        );
        let stuck = spec.recovery / 3;
        let end = strike::frames(spec);
        let frames = [
            0,
            spec.startup / 3,
            spec.startup - 1,
            spec.startup + spec.active / 2,
            spec.startup + spec.active - 1,
            spec.startup + spec.active + stuck - 1,
            end - 1,
        ];
        for fr in frames {
            println!("frame {fr} {:?}", strike::beat(spec, fr));
            print_boxes(&strike::body_at(&g, &q, m, spec, fr));
            // Every strike frame's spheres, drawn on the strike's frames so
            // the whole sweep shows.
            if matches!(strike::beat(spec, fr), strike::Beat::Strike) {
                for sf in spec.startup..=fr {
                    for h in strike::hit_at(&g, &q, m, spec, sf).iter() {
                        println!(
                            "hit {:.3} {:.3} {:.3} {:.3} {}",
                            f(h.at.x),
                            f(h.at.y),
                            f(h.at.z),
                            f(h.radius),
                            if sf == fr { 1 } else { 0 }
                        );
                    }
                }
            }
        }
    }
}

/// Every gait of a creature as eight moments of its cycle, and its
/// footfall pattern, for `scripts/breed-motion.py --gaits`.
///
/// ```text
/// gait <kind> <froude> <speed> <stride> <cycle> <duty>
/// foot <station> <side> <offset>
/// frame <phase>
/// obox ...
/// ```
fn strides(args: &[String]) {
    use sim::breed::{locomotion, posture};
    let g = args
        .first()
        .and_then(|n| by_name(n))
        .copied()
        .unwrap_or(RIDGEBACK);
    let q = posture::posture(&g);
    for gait in locomotion::repertoire(&g, &q).into_iter().flatten() {
        println!(
            "gait {} {:.2} {:.1} {:.2} {} {:.2}",
            gait.kind.name(),
            f(gait.froude),
            f(gait.speed),
            f(gait.stride),
            gait.cycle,
            f(gait.duty)
        );
        for (i, walks) in gait.walks.iter().enumerate() {
            if *walks {
                for side in 0..2 {
                    println!("foot {i} {side} {:.3}", f(gait.offset[i][side]));
                }
            }
        }
        for k in 0..8 {
            let phase = Fx::ratio(k, 8);
            println!("frame {:.3}", f(phase));
            let d = locomotion::drive(&g, &q, &gait, phase, Fx::ZERO);
            print_boxes(&build::body_driven(&g, &q, &d));
        }
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
        print_boxes(&body);
    }
}

/// Every move of a cast member (or all of them), measured against the
/// bestiary contract: its tell, how much and how early the silhouette
/// moves, the lane and range its hit reaches, its answer, and the kit's
/// totals.
fn moves_report(only: Option<&str>) {
    use sim::breed::{posture, verdict};
    for (_, name, g, _) in CAST {
        if only.is_some_and(|o| !name.to_lowercase().starts_with(&o.to_lowercase())) {
            continue;
        }
        let q = posture::posture(g);
        let v = verdict::judge(g, &q);
        let len = build::proportions(g).len;
        println!("== {name} (length {:.1} m)", f(len));
        for r in v.iter() {
            let flag = if r.unanswerable(len) {
                "  UNANSWERABLE"
            } else if !r.damaging {
                "  (no hit)"
            } else {
                ""
            };
            println!(
                "   {:<15} tell {:>3} f  moves {:>5.2} m ({:>3}% by a third)  {:<6} {:<5} {:>5.1} m  swing {:>5.2} m  stuck {:>2}/{:>3} f  -> {}{flag}",
                r.kind.name(),
                r.tell,
                f(r.change),
                (r.early.raw() as i64 * 100) >> 16,
                r.lane.name(),
                r.range.name(),
                f(r.beyond),
                f(r.swing),
                r.stuck,
                r.recovery,
                r.answer.name()
            );
        }
        println!(
            "   {} damaging, {} questions, {} answers, {} lookalike pairs, {} unanswerable, {} safe; ranges {}{}{}{}",
            v.damaging,
            v.questions,
            v.answers,
            v.lookalikes,
            v.unanswerable,
            v.safe,
            if v.close { "close " } else { "" },
            if v.mid { "mid " } else { "" },
            if v.long { "long " } else { "" },
            match v.aboard {
                Some(true) => "aboard",
                Some(false) => "(nothing aboard)",
                None => "",
            }
        );
        for (a, b) in v.lookalike.iter().flatten() {
            println!("   lookalike: {} and {} cock the same", a.name(), b.name());
        }
    }
}

/// Every cast member's gaits, slowest first: what it changes up to as it
/// goes faster, at what Froude number and speed, its stride and cadence.
fn gaits() {
    use sim::breed::{locomotion, posture};
    for (_, name, g, _) in CAST {
        let q = posture::posture(g);
        print!(
            "{name:<14} hips {:.2} m, top Fr {:.2}:",
            f(q.root),
            f(locomotion::top_froude(g, &q))
        );
        for gait in locomotion::repertoire(g, &q).into_iter().flatten() {
            print!(
                "  {} (Fr {:.2}, {:.1} m/s, stride {:.2} m, {} f)",
                gait.kind.name(),
                f(gait.froude),
                f(gait.speed),
                f(gait.stride),
                gait.cycle
            );
        }
        println!();
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
        Some("gaits") => gaits(),
        Some("film") => film(&args[1..]),
        Some("strides") => strides(&args[1..]),
        Some("moves") => moves_report(args.get(1).map(String::as_str)),
        Some(other) => println!("unknown: {other}"),
    }
}
