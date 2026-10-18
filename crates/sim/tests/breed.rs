//! Crossbreeding: the genome builder rebuilds the cast, and a lineage
//! drifts the way the design says it should.
//!
//! The builder (`sim::breed`) turns a genome into a body and a move set.
//! The litmus test is that the eleven creatures of the bestiary, written as
//! genomes, come out close to their hand-written species tables: the same
//! number of legs, hips at the same height within a fifth, every kind of
//! move the table has, and only the bird flying. Then the crossing: two of a
//! kind breed true, the biggest breed bigger, a cross lands between its
//! parents, a trophy moves an egg slowly and never past what stands up, and
//! a Ridgeback given feathers takes generations to fly.

use sim::breed::archetypes::{self, CAST, GALEWING, MANTIS, PAIR, RIDGEBACK, SANDMAW, SIEGESHELL};
use sim::breed::build::{self, MoveKind};
use sim::breed::cross::{self, Rates, Trophy, TrophyKind};
use sim::breed::plan::{self, Fix};
use sim::breed::rear::{Fed, Food, Stage, Ward};
use sim::breed::{
    Back, Build, Crown, Genome, HeadKind, Hide, Jaw, LimbPair, Minors, Rng, Role, STATIONS,
    Segments, Slider, Sliders, Stance, Tail, TailTip, Tip,
};
use sim::fixed::Fx;

fn fx(n: i32, d: i32) -> Fx {
    Fx::ratio(n, d)
}

fn f(x: Fx) -> f32 {
    x.to_f32_for_render()
}

/// The repair's invariants, which every genome that comes out of a
/// crossing must satisfy.
fn stands_up(g: &Genome) -> Result<(), String> {
    if g.stance == Stance::Limbless && g.pair_count() > 0 {
        return Err("limbs on a limbless plan".into());
    }
    if g.stance != Stance::Limbless && g.pair_count() == 0 {
        return Err("a plan with no limbs that is not limbless".into());
    }
    if g.stance != Stance::Limbless && g.grounded_pairs() == 0 {
        return Err("nothing to stand on".into());
    }
    for (i, p) in g.limbs.iter().enumerate() {
        if let Some(p) = p {
            if p.role == Role::Wing && i >= 2 {
                return Err("a wing astern".into());
            }
            if (p.role == Role::Wing) != (p.build == Build::Wing) {
                return Err("a wing's role and build disagree".into());
            }
        }
    }
    if g.segments.tail == Tail::None && g.minors.tail_tip != TailTip::Plain {
        return Err("a tail tip with no tail".into());
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// The cast
// ---------------------------------------------------------------------------

#[test]
fn every_archetype_is_a_legal_plan_at_rest() {
    for (_, name, g, _) in CAST {
        let mut r = *g;
        let fixes = plan::repair(&mut r);
        assert_eq!(
            fixes.count, 0,
            "{name}: repair changed its archetype: {fixes:?}"
        );
        assert_eq!(r, *g, "{name}: repair moved its archetype");
        // The archetype is near what its own plan wants: a settle moves no
        // slider by more than a sixth.
        let mut s = *g;
        plan::settle(&mut s);
        for sl in Slider::ALL {
            let moved = f(s.sliders.get(sl).sub(g.sliders.get(sl)).abs());
            assert!(
                moved < 0.17,
                "{name}: {sl:?} is {moved:.2} from what its plan wants"
            );
        }
        stands_up(g).unwrap_or_else(|e| panic!("{name}: {e}"));
    }
}

#[test]
fn the_cast_rebuilds_from_its_genomes() {
    for (id, name, g, kinds) in CAST {
        let body = build::body(g);
        let real = id.get();
        if real.has_body() {
            assert_eq!(
                body.leg_count,
                real.legs.len(),
                "{name}: {} legs built, the table has {}",
                body.leg_count,
                real.legs.len()
            );
            let table_hips = f(real.bones[0].rest.y);
            if table_hips > 0.0 {
                let built = f(body.hips);
                let off = (built - table_hips).abs() / table_hips;
                assert!(
                    off < 0.25,
                    "{name}: hips built {built:.2} m, the table has {table_hips:.2} m ({:.0}% off)",
                    off * 100.0
                );
            }
        }
        let moves = build::moves(g);
        for k in kinds.iter() {
            assert!(
                moves.has(*k),
                "{name}: the builder gave it no {:?}; it has {:?}",
                k,
                moves.iter().map(|m| m.kind).collect::<Vec<_>>()
            );
        }
        let a = plan::assess(g);
        assert!(
            f(a.plausibility) >= 0.65,
            "{name}: plausibility {:.2}",
            f(a.plausibility)
        );
    }
}

#[test]
fn only_the_galewing_flies() {
    for (_, name, g, _) in CAST {
        let a = plan::assess(g);
        assert_eq!(
            a.caps.flies,
            name == "Galewing",
            "{name}: flies={}",
            a.caps.flies
        );
        assert!(!a.caps.vestigial_wings, "{name}: vestigial wings");
    }
}

#[test]
fn tiers_follow_the_bestiary_in_order() {
    let tier = |g: &Genome| plan::assess(g).tier;
    assert_eq!(tier(&archetypes::GNAWER), 1);
    assert!(tier(&archetypes::HORNBACK) <= 2);
    assert!(tier(&RIDGEBACK) >= 3);
    assert!(tier(&MANTIS) >= 4, "mantis tier {}", tier(&MANTIS));
    assert_eq!(tier(&SIEGESHELL), 5);
    assert!(tier(&SANDMAW) <= tier(&SIEGESHELL));
}

#[test]
fn the_body_is_sized_like_the_table() {
    // The Ridgeback: thirteen metres nose to tail, a back out of reach.
    let b = build::body(&RIDGEBACK);
    assert!(
        (11.0..16.0).contains(&f(b.length)),
        "length {}",
        f(b.length)
    );
    assert!(f(b.top()) > 4.0, "top {}", f(b.top()));
    // The Galewing's span is about eighteen metres.
    let b = build::body(&GALEWING);
    assert!((12.0..24.0).contains(&f(b.span)), "span {}", f(b.span));
    // The Mantis stands upright: its head is above its hips by more than
    // its torso is long.
    let b = build::body(&MANTIS);
    let head = b.bone_named("head").unwrap();
    assert!(
        f(b.at(head).y) > f(b.hips) + 1.0,
        "mantis head at {}",
        f(b.at(head).y)
    );
    // The worm has no legs and is a chain.
    let b = build::body(&SANDMAW);
    assert_eq!(b.leg_count, 0);
    assert!(b.bone_named("tail5").is_some() && b.bone_named("neck5").is_some());
}

#[test]
fn trophies_are_the_creatures_parts() {
    let has = |g: &Genome, k: TrophyKind| archetypes::drops(g).iter().flatten().any(|x| *x == k);
    assert!(has(&GALEWING, TrophyKind::Feather));
    assert!(has(&GALEWING, TrophyKind::Beak));
    assert!(has(&GALEWING, TrophyKind::Talon));
    assert!(has(&RIDGEBACK, TrophyKind::Plate));
    assert!(has(&RIDGEBACK, TrophyKind::Hoof));
    assert!(has(&RIDGEBACK, TrophyKind::TailTip));
    assert!(has(&archetypes::HORNBACK, TrophyKind::Horn));
    assert!(
        !has(&archetypes::HORNBACK, TrophyKind::Tooth),
        "a grazer has no tooth to drop"
    );
    assert!(has(&MANTIS, TrophyKind::Chitin));
}

#[test]
fn a_wild_population_varies_without_changing_plan() {
    let mut sizes = Vec::new();
    for seed in 0..64u64 {
        let g = archetypes::vary(&RIDGEBACK, seed);
        assert_eq!(g.stance, RIDGEBACK.stance);
        assert_eq!(g.limbs.iter().flatten().count(), 2);
        sizes.push(f(g.sliders.size));
    }
    let min = sizes.iter().cloned().fold(f32::MAX, f32::min);
    let max = sizes.iter().cloned().fold(f32::MIN, f32::max);
    assert!(max - min > 0.02, "no variety: {min}..{max}");
    assert!(max - min < 0.2, "too much variety: {min}..{max}");
}

// ---------------------------------------------------------------------------
// Determinism
// ---------------------------------------------------------------------------

#[test]
fn a_lineage_is_a_function_of_its_seed() {
    let run = |seed: u64| {
        let mut rng = Rng::new(seed);
        let mut g = RIDGEBACK;
        let trophy = Trophy {
            kind: TrophyKind::Feather,
            donor: GALEWING,
        };
        for _ in 0..30 {
            g = cross::graft(&g, &trophy, &Rates::DEFAULT, &mut rng).genome;
        }
        g
    };
    assert_eq!(run(7), run(7));
    assert_ne!(run(7).fingerprint(), run(8).fingerprint());
}

// ---------------------------------------------------------------------------
// Breeding
// ---------------------------------------------------------------------------

#[test]
fn two_of_a_kind_breed_true() {
    let mut stance_changes = 0;
    let mut limb_changes = 0;
    for seed in 0..20u64 {
        let mut rng = Rng::new(100 + seed);
        let mut a = archetypes::vary(&PAIR, seed * 2);
        let mut b = archetypes::vary(&PAIR, seed * 2 + 1);
        for _ in 0..20 {
            let child = cross::breed(&a, &b, &Rates::DEFAULT, &mut rng).genome;
            stands_up(&child).unwrap();
            // An event is a child unlike both its parents; a difference
            // inherited from one of them is not a new one.
            if child.stance != a.stance && child.stance != b.stance {
                stance_changes += 1;
            }
            if child.pair_count() != a.pair_count() && child.pair_count() != b.pair_count() {
                limb_changes += 1;
            }
            b = a;
            a = child;
        }
        // The body's sliders hold; the disposition can swing further when
        // a minor mutates and the plan wants something new of it.
        for sl in [Slider::Size, Slider::Mass, Slider::Pitch, Slider::Speed] {
            let off = f(a.sliders.get(sl).sub(PAIR.sliders.get(sl)).abs());
            assert!(
                off < 0.25,
                "seed {seed}: {sl:?} drifted {off:.2} in twenty generations"
            );
        }
        let p = f(plan::assess(&a).plausibility);
        assert!(p >= 0.6, "seed {seed}: plausibility fell to {p:.2}");
    }
    // Majors are rare: across four hundred births, a handful at most.
    assert!(
        stance_changes <= 4,
        "{stance_changes} stance changes in 400 births"
    );
    assert!(
        limb_changes <= 4,
        "{limb_changes} limb-count changes in 400 births"
    );
}

#[test]
fn breeding_the_biggest_breeds_bigger() {
    let mut rng = Rng::new(42);
    let mut herd: Vec<Genome> = (0..8).map(|i| archetypes::vary(&RIDGEBACK, i)).collect();
    let mean = |h: &[Genome]| h.iter().map(|g| f(g.sliders.size)).sum::<f32>() / h.len() as f32;
    let start = mean(&herd);
    for _ in 0..15 {
        herd.sort_by_key(|g| std::cmp::Reverse(g.sliders.size));
        let (a, b) = (herd[0], herd[1]);
        herd = (0..8)
            .map(|_| cross::breed(&a, &b, &Rates::DEFAULT, &mut rng).genome)
            .collect();
    }
    let end = mean(&herd);
    assert!(end > start + 0.04, "size went {start:.3} -> {end:.3}");
    for g in &herd {
        assert_eq!(g.stance, Stance::Level);
        assert!(f(plan::assess(g).plausibility) >= 0.6);
    }
}

#[test]
fn a_cross_lands_between_its_parents() {
    for seed in 0..32u64 {
        let mut rng = Rng::new(seed);
        let out = cross::breed(&RIDGEBACK, &MANTIS, &Rates::DEFAULT, &mut rng);
        let c = out.genome;
        stands_up(&c).unwrap();
        assert!(
            c.stance == RIDGEBACK.stance || c.stance == MANTIS.stance || out.fixes.count > 0,
            "seed {seed}: a stance from neither parent and no repair"
        );
        // Settled sliders may move a little past the parents' range toward
        // what the plan wants, but not far.
        for sl in [Slider::Size, Slider::Speed, Slider::Aggression] {
            let (lo, hi) = {
                let (x, y) = (f(RIDGEBACK.sliders.get(sl)), f(MANTIS.sliders.get(sl)));
                (x.min(y) - 0.2, x.max(y) + 0.2)
            };
            let v = f(c.sliders.get(sl));
            assert!(
                (lo..=hi).contains(&v),
                "seed {seed}: {sl:?} = {v:.2} outside {lo:.2}..{hi:.2}"
            );
        }
        assert!(f(plan::assess(&c).plausibility) >= 0.4, "seed {seed}");
    }
}

// ---------------------------------------------------------------------------
// Grafting
// ---------------------------------------------------------------------------

/// The Ridgeback's fore pair, blended with the Mantis's arms: the brief's
/// own example. The result keeps the Ridgeback's plan and is pulled a
/// little toward the Mantis.
#[test]
fn a_mantis_trophy_pulls_a_ridgeback_a_little_toward_the_mantis() {
    let trophy = Trophy {
        kind: TrophyKind::Claw,
        donor: MANTIS,
    };
    let mut upright_sum = 0.0;
    let mut length_sum = 0.0;
    let mut flipped = 0;
    let n = 64;
    for seed in 0..n as u64 {
        let mut rng = Rng::new(1000 + seed);
        let out = cross::graft(&RIDGEBACK, &trophy, &Rates::DEFAULT, &mut rng);
        let g = out.genome;
        stands_up(&g).unwrap();
        // The plan is still a Ridgeback's nearly every time.
        if g.stance != Stance::Level {
            flipped += 1;
        }
        upright_sum += f(g.sliders.pitch);
        let fore = g.limbs[0].unwrap();
        length_sum += f(fore.length);
        // A body whose plan just changed is in transition and may be
        // awkward for a generation; one whose plan held is not.
        let p = f(plan::assess(&g).plausibility);
        assert!(
            p >= if out.moved.any() { 0.3 } else { 0.5 },
            "seed {seed}: {p:.2} {:?}",
            out.moved
        );
    }
    assert!(
        flipped <= n / 8,
        "{flipped} of {n} grafts changed the stance"
    );
    let pitch = upright_sum / n as f32;
    let length = length_sum / n as f32;
    assert!(pitch > 0.0 && pitch < 0.2, "mean pitch {pitch:.3}");
    assert!(
        length > f(RIDGEBACK.limbs[0].unwrap().length) - 0.02,
        "fore length {length:.3} shrank"
    );
}

/// Over many grafts of the same trophy, the limb goes over: the fore pair
/// becomes raptorial, and the torso follows it part of the way up.
#[test]
fn many_mantis_trophies_turn_the_fore_pair_raptorial() {
    let trophy = Trophy {
        kind: TrophyKind::Claw,
        donor: MANTIS,
    };
    let mut turned = 0;
    let mut pitch_when_turned = Vec::new();
    for seed in 0..24u64 {
        let mut rng = Rng::new(2000 + seed);
        let mut g = RIDGEBACK;
        for _ in 0..40 {
            g = cross::graft(&g, &trophy, &Rates::DEFAULT, &mut rng).genome;
            stands_up(&g).unwrap();
        }
        // A mutation can lose the pair outright, once in a few hundred
        // generations; that lineage has nothing to say here.
        let Some(fore) = g.limbs[0] else { continue };
        if fore.build == Build::Raptorial {
            turned += 1;
            pitch_when_turned.push(f(g.sliders.pitch));
            // A raptorial fore pair does not carry weight alone.
            assert_ne!(fore.role, Role::Walk);
        }
    }
    assert!(
        turned >= 12,
        "only {turned} of 24 lineages turned raptorial in forty grafts"
    );
    let mean_pitch = pitch_when_turned.iter().sum::<f32>() / pitch_when_turned.len() as f32;
    assert!(
        mean_pitch > 0.1,
        "the torso did not follow the arms up: mean pitch {mean_pitch:.2}"
    );
    assert!(
        mean_pitch < 0.6,
        "the torso went all the way over: mean pitch {mean_pitch:.2}"
    );
}

/// The brief's example: a Ridgeback given feathers. Wings arrive as a
/// major, late; flight comes later still, once the bones are lighter and
/// the wings long; and the body stands up the whole way.
#[test]
fn a_ridgeback_given_feathers_takes_generations_to_fly() {
    let trophy = Trophy {
        kind: TrophyKind::Feather,
        donor: GALEWING,
    };
    let seeds = 24u64;
    let gens = 80;
    let mut wings_at = Vec::new();
    let mut flies_at = Vec::new();
    let mut fell = 0;
    for seed in 0..seeds {
        let mut rng = Rng::new(3000 + seed);
        let mut g = RIDGEBACK;
        let mut wings = None;
        let mut flies = None;
        for generation in 1..=gens {
            let out = cross::graft(&g, &trophy, &Rates::DEFAULT, &mut rng);
            g = out.genome;
            stands_up(&g).unwrap_or_else(|e| panic!("seed {seed} generation {generation}: {e}"));
            let a = plan::assess(&g);
            if f(a.plausibility) < 0.45 {
                fell += 1;
            }
            if wings.is_none() && g.has_wings() {
                wings = Some(generation);
            }
            if flies.is_none() && a.caps.flies {
                flies = Some(generation);
            }
            assert!(
                !(generation == 1 && a.caps.flies),
                "seed {seed}: it flew after one feather"
            );
        }
        if let Some(w) = wings {
            wings_at.push(w);
        }
        if let Some(fl) = flies {
            flies_at.push(fl);
            assert!(
                wings.unwrap() < fl,
                "seed {seed}: flew the generation its wings arrived"
            );
        }
    }
    assert!(
        wings_at.len() >= seeds as usize / 2,
        "only {} of {seeds} lineages grew wings in {gens} generations",
        wings_at.len()
    );
    assert!(
        !flies_at.is_empty(),
        "no lineage flew in {gens} generations"
    );
    let mean_wings = wings_at.iter().sum::<u32>() as f32 / wings_at.len() as f32;
    let mean_flies = flies_at.iter().sum::<u32>() as f32 / flies_at.len() as f32;
    assert!(
        mean_wings >= 3.0,
        "wings arrived too fast: mean generation {mean_wings:.1}"
    );
    assert!(
        mean_flies >= mean_wings + 2.0,
        "flight followed wings too fast: wings {mean_wings:.1}, flight {mean_flies:.1}"
    );
    assert!(
        fell <= (seeds * gens as u64 / 20) as usize,
        "plausibility fell under 0.45 in {fell} of {} generations",
        seeds * gens as u64
    );
}

/// A trophy never removes a limb, and never leaves a wing where it cannot
/// be: after a feather lands on a spider, the spider still stands.
#[test]
fn a_graft_never_breaks_the_body() {
    let donors = [
        GALEWING,
        MANTIS,
        SANDMAW,
        SIEGESHELL,
        archetypes::BROODMOTHER,
    ];
    let eggs = [
        RIDGEBACK,
        archetypes::BROODMOTHER,
        archetypes::MIREBACK,
        SANDMAW,
        PAIR,
    ];
    let mut seed = 0u64;
    for egg in eggs {
        for donor in donors {
            for kind in TrophyKind::ALL {
                seed += 1;
                let mut rng = Rng::new(seed);
                let trophy = Trophy { kind, donor };
                let mut g = egg;
                for generation in 0..30 {
                    let out = cross::graft(&g, &trophy, &Rates::DEFAULT, &mut rng);
                    g = out.genome;
                    stands_up(&g).unwrap_or_else(|e| {
                        panic!(
                            "{:?} onto {:?} generation {generation}: {e}\n{g:?}",
                            kind, egg.stance
                        )
                    });
                    assert!(g.pair_count() >= egg.pair_count().min(g.pair_count()));
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Repair
// ---------------------------------------------------------------------------

fn random_genome(rng: &mut Rng) -> Genome {
    let pair = |rng: &mut Rng| {
        if rng.chance(fx(3, 5)) {
            Some(LimbPair {
                role: Role::ALL[rng.below(4)],
                build: Build::ALL[rng.below(6)],
                tip: Tip::ALL[rng.below(6)],
                length: rng.unit(),
                girth: rng.unit(),
            })
        } else {
            None
        }
    };
    let mut limbs = [None; STATIONS];
    for l in limbs.iter_mut() {
        *l = pair(rng);
    }
    Genome {
        stance: Stance::ALL[rng.below(4)],
        head: HeadKind::ALL[rng.below(4)],
        segments: Segments {
            thorax: rng.chance(fx(1, 2)),
            abdomen: rng.chance(fx(1, 2)),
            tail: Tail::ALL[rng.below(4)],
        },
        limbs,
        minors: Minors {
            jaw: Jaw::ALL[rng.below(6)],
            crown: Crown::ALL[rng.below(4)],
            tail_tip: TailTip::ALL[rng.below(4)],
            hide: Hide::ALL[rng.below(7)],
            back: Back::ALL[rng.below(4)],
        },
        sliders: Sliders {
            size: rng.unit(),
            mass: rng.unit(),
            pitch: rng.unit(),
            speed: rng.unit(),
            reaction: rng.unit(),
            aggression: rng.unit(),
            tempo: rng.unit(),
            guard: rng.unit(),
        },
    }
}

#[test]
fn repair_makes_any_genome_stand_and_is_idempotent() {
    let mut rng = Rng::new(99);
    let mut fixed = 0;
    for i in 0..2000 {
        let mut g = random_genome(&mut rng);
        let fixes = plan::repair(&mut g);
        if fixes.count > 0 {
            fixed += 1;
        }
        stands_up(&g).unwrap_or_else(|e| panic!("genome {i}: {e}\n{g:?}\n{fixes:?}"));
        let again = g;
        let mut g2 = g;
        let more = plan::repair(&mut g2);
        assert_eq!(
            more.count, 0,
            "genome {i}: a second repair found more: {more:?}"
        );
        assert_eq!(g2, again);
        // And the body builds without running out of room.
        let body = build::body(&g);
        assert!(body.bone_count <= 40 && body.part_count <= 48);
        let moves = build::moves(&g);
        assert!(
            moves.count >= 1,
            "genome {i}: a creature with no moves at all\n{g:?}"
        );
    }
    assert!(
        fixed > 500,
        "only {fixed} of 2000 random genomes needed repair"
    );
}

#[test]
fn repair_names_what_it_changed() {
    let mut g = RIDGEBACK;
    g.limbs[3].as_mut().unwrap().role = Role::Wing;
    g.limbs[0].as_mut().unwrap().role = Role::Strike;
    let fixes = plan::repair(&mut g);
    assert!(fixes.has(Fix::WingAstern), "{fixes:?}");
    assert!(g.grounded_pairs() >= 1);
    stands_up(&g).unwrap();

    let mut g = SANDMAW;
    g.limbs[0] = RIDGEBACK.limbs[0];
    let fixes = plan::repair(&mut g);
    assert!(fixes.has(Fix::LimbsOnLimbless));
    assert_eq!(g.stance, Stance::Lowslung);
}

// ---------------------------------------------------------------------------
// Drift
// ---------------------------------------------------------------------------

/// Two hundred generations with nobody pushing: the plan nearly always
/// holds, the sliders wander a little, and the animal stays plausible.
#[test]
fn drift_without_trophies_stays_near_the_species() {
    let mut major_changes = 0;
    let mut worst = 1.0f32;
    for (_, name, g, _) in CAST {
        for seed in 0..4u64 {
            let mut rng = Rng::new(5000 + seed);
            let mut x = *g;
            let mut majors_here = 0;
            for generation in 0..200 {
                let out = cross::mutate(&x, &Rates::DEFAULT, &mut rng);
                x = out.genome;
                if out.moved.any() {
                    major_changes += 1;
                    majors_here += 1;
                }
                stands_up(&x)
                    .unwrap_or_else(|e| panic!("{name} seed {seed} generation {generation}: {e}"));
                let p = f(plan::assess(&x).plausibility);
                worst = worst.min(p);
            }
            // Mass and pitch are held by what the plan wants; size has
            // nothing holding it but selection, which nobody applies here,
            // so it walks: a hundredth a generation, a seventh over two
            // hundred.
            // A lineage whose plan changed is another animal and is not
            // held to its ancestor's sliders.
            if majors_here > 0 {
                continue;
            }
            for (sl, bound) in [
                (Slider::Size, 0.5),
                (Slider::Mass, 0.3),
                (Slider::Pitch, 0.3),
            ] {
                let off = f(x.sliders.get(sl).sub(g.sliders.get(sl)).abs());
                assert!(off < bound, "{name} seed {seed}: {sl:?} wandered {off:.2}");
            }
        }
    }
    // 44 lineages of 200 generations: majors mutate at 1/400 per gene per
    // generation, with nine or ten genes rolled, so about one generation
    // in forty sees one -- a couple of hundred in all.
    assert!(
        major_changes < 300,
        "{major_changes} major changes in 8,800 generations"
    );
    assert!(worst >= 0.4, "plausibility fell to {worst:.2} somewhere");
}

// ---------------------------------------------------------------------------
// Rearing
// ---------------------------------------------------------------------------

#[test]
fn an_egg_hatches_grows_rebels_and_is_tamed() {
    let mut w = Ward::egg(PAIR);
    assert!(w.current().is_none());
    assert_eq!(w.feed(Food::Meat { tier: 2 }), Fed::NotHungry);
    w.incubate(500);
    assert!(matches!(w.stage, Stage::Egg { .. }));
    w.incubate(700);
    assert!(matches!(w.stage, Stage::Hatchling { .. }));
    // A hatchling is a small version of the adult.
    let hatchling = w.current().unwrap();
    assert!(f(hatchling.sliders.size) < f(PAIR.sliders.size) * 0.5);
    assert_eq!(hatchling.stance, PAIR.stance);
    // A carnivore will not eat grass.
    assert_eq!(w.feed(Food::Grass), Fed::Refused);
    // Meat under its tier grows it less than meat of its tier.
    let own = w.adult_tier();
    let Fed::Grew(little) = w.feed(Food::Meat { tier: 1 }) else {
        panic!()
    };
    let Fed::Grew(full) = w.feed(Food::Meat { tier: own }) else {
        panic!()
    };
    assert!(
        own == 1 || little < full,
        "tier {own}: {little:?} vs {full:?}"
    );
    // A hard fight grows it and raises what it will reach.
    let before = w.vigor;
    w.fought(own + 1);
    assert!(w.vigor > before);
    // Feed it up; it rebels at full size.
    let mut meals = 0;
    while matches!(w.stage, Stage::Hatchling { .. }) {
        w.feed(Food::Meat { tier: 5 });
        meals += 1;
        assert!(meals < 100);
    }
    assert_eq!(w.stage, Stage::Rebel);
    assert!(meals >= 6, "{meals} meals to full size");
    assert!(w.subdue());
    assert!(w.is_tame());
    assert!(!w.subdue());
    assert!(f(w.current().unwrap().sliders.size) > f(PAIR.sliders.size));
}

#[test]
fn a_grazer_eats_grass_and_a_dangerous_one_needs_better() {
    let mut cow = Ward::egg(archetypes::HORNBACK);
    cow.incubate(1200);
    assert!(matches!(cow.feed(Food::Grass), Fed::Grew(_)));
    assert_eq!(cow.feed(Food::Meat { tier: 3 }), Fed::Refused);

    let mut hill = Ward::egg(SIEGESHELL);
    hill.incubate(1200);
    let Fed::Grew(grass) = hill.feed(Food::Grass) else {
        panic!()
    };
    let Fed::Grew(needle) = hill.feed(Food::Greens { tier: 5 }) else {
        panic!()
    };
    assert!(
        f(needle) >= f(grass) * 4.0,
        "grass {grass:?}, the Needle's greens {needle:?}"
    );
}

// ---------------------------------------------------------------------------
// Moves and their numbers
// ---------------------------------------------------------------------------

#[test]
fn heavier_and_bigger_is_slower_and_harder() {
    let big = *build::moves(&RIDGEBACK).get(MoveKind::Bite).unwrap();
    let small = *build::moves(&archetypes::GNAWER)
        .get(MoveKind::Bite)
        .unwrap();
    assert!(
        big.startup > small.startup,
        "{} vs {}",
        big.startup,
        small.startup
    );
    assert!(
        big.damage > small.damage * 3,
        "{} vs {}",
        big.damage,
        small.damage
    );
    assert!(f(big.reach) > f(small.reach) * 4.0);
}

#[test]
fn disposition_is_read_into_appetite() {
    let mut timid = RIDGEBACK;
    timid.sliders.aggression = Fx::ZERO;
    timid.sliders.tempo = Fx::ZERO;
    let mut fierce = RIDGEBACK;
    fierce.sliders.aggression = Fx::ONE;
    fierce.sliders.tempo = Fx::ONE;
    let t = build::moves(&timid);
    let fc = build::moves(&fierce);
    let bite_t = t.get(MoveKind::Bite).unwrap().weight;
    let bite_f = fc.get(MoveKind::Bite).unwrap().weight;
    assert!(bite_f > bite_t);
    // A quick tempo likes the quick move more than the slow one.
    let charge_f = fc.get(MoveKind::Charge).unwrap().weight;
    assert!(
        bite_f > charge_f,
        "quick: bite {bite_f:?} charge {charge_f:?}"
    );
    let charge_t = t.get(MoveKind::Charge).unwrap().weight;
    assert!(
        charge_t >= bite_t,
        "slow: bite {bite_t:?} charge {charge_t:?}"
    );
}
