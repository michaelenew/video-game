//! How crossbred creatures move: gaits chosen as animals choose them, feet
//! that stay where they are put, and moves that answer the bestiary
//! contract (`docs/design/bestiary.md` §1) -- for the cast, and for every
//! generation of lineages bred from it.

use sim::breed::archetypes::CAST;
use sim::breed::build::proportions;
use sim::breed::cross::{self, Rates, Trophy, TrophyKind};
use sim::breed::locomotion::{self, GaitKind};
use sim::breed::verdict::{self, Lane};
use sim::breed::{Genome, Rng, posture};
use sim::fixed::Fx;

const fn fx(num: i32, den: i32) -> Fx {
    Fx::ratio(num, den)
}

fn cast(name: &str) -> &'static Genome {
    CAST.iter().find(|c| c.1 == name).map(|c| c.2).unwrap()
}

/// The contract, as a crossbred body has to meet it: nothing unanswerable,
/// nothing safe, no two windups alike, and every hit low enough to touch
/// somebody standing.
fn holds(g: &Genome) -> Result<(), String> {
    let q = posture::posture(g);
    let v = verdict::judge(g, &q);
    let len = proportions(g).len;
    for r in v.iter().filter(|r| r.damaging) {
        if r.unanswerable(len) {
            return Err(format!(
                "{} is unanswerable: a {}-frame tell, {:?} m of change",
                r.kind.name(),
                r.tell,
                r.change
            ));
        }
        if r.lane == Lane::Above {
            return Err(format!("{} passes over a standing head", r.kind.name()));
        }
    }
    if v.safe > 0 {
        return Err(format!("{} moves are safe to throw", v.safe));
    }
    if v.lookalikes > 0 {
        return Err(format!("windups alike: {:?}", v.lookalike[0]));
    }
    Ok(())
}

#[test]
fn the_cast_answers_the_contract() {
    for (_, name, g, _) in CAST {
        if let Err(e) = holds(g) {
            panic!("{name}: {e}");
        }
        let q = posture::posture(g);
        let v = verdict::judge(g, &q);
        assert!(
            v.questions >= 3,
            "{name} asks only {} questions",
            v.questions
        );
        assert!(v.close, "{name} has nothing up close");
    }
}

#[test]
fn crossbred_generations_answer_the_contract() {
    // Drift, a trophy every fourth generation, and one every generation:
    // whatever the body becomes, its moves can be read and answered.
    let lines = [
        ("Ridgeback", None, 0),
        ("Ridgeback", Some((TrophyKind::Claw, "Mantis")), 1),
        ("Ridgeback", Some((TrophyKind::Feather, "Galewing")), 1),
        ("Mantis", Some((TrophyKind::Fang, "Veilstalker")), 4),
        ("Broodmother", Some((TrophyKind::Horn, "Hornback")), 4),
        ("Sandmaw", None, 0),
        ("Gnawers", Some((TrophyKind::Feather, "Galewing")), 1),
    ];
    for (k, (egg, trophy, every)) in lines.into_iter().enumerate() {
        let mut rng = Rng::new(17 + k as u64);
        let mut g = *cast(egg);
        for generation in 1..=30 {
            g = match trophy {
                Some((kind, donor)) if every > 0 && generation % every == 0 => {
                    let t = Trophy {
                        kind,
                        donor: *cast(donor),
                    };
                    cross::graft(&g, &t, &Rates::DEFAULT, &mut rng).genome
                }
                _ => cross::mutate(&g, &Rates::DEFAULT, &mut rng).genome,
            };
            if let Err(e) = holds(&g) {
                panic!("{egg} line {k}, generation {generation}: {e}");
            }
        }
    }
}

#[test]
fn a_body_changes_gait_as_animals_do() {
    let gaits = |name: &str| {
        let g = cast(name);
        let q = posture::posture(g);
        locomotion::repertoire(g, &q)
            .into_iter()
            .flatten()
            .map(|x| x.kind)
            .collect::<Vec<_>>()
    };
    // Four legs walk, then trot; eight wave, then go four feet at a time;
    // none slither.
    assert_eq!(gaits("Ridgeback")[..2], [GaitKind::Walk, GaitKind::Trot]);
    assert_eq!(gaits("Broodmother"), [GaitKind::Wave, GaitKind::Tetrapod]);
    assert_eq!(gaits("Sandmaw"), [GaitKind::Slither]);
    // The walk to trot change is at the same Froude number whatever the
    // size: a gnawer and the Ridgeback change at the same place.
    for name in ["Ridgeback", "Hornback", "Gnawers"] {
        let g = cast(name);
        let q = posture::posture(g);
        let trot = locomotion::repertoire(g, &q)
            .into_iter()
            .flatten()
            .find(|x| x.kind == GaitKind::Trot)
            .unwrap();
        assert!(trot.froude >= fx(1, 2), "{name} trots at {:?}", trot.froude);
    }
}

#[test]
fn a_big_body_walks_with_a_slower_cadence() {
    let walk = |name: &str| {
        let g = cast(name);
        locomotion::walking(g, &posture::posture(g))
    };
    let (big, small) = (walk("Ridgeback"), walk("Gnawers"));
    assert!(
        big.cycle > small.cycle * 2,
        "{} vs {}",
        big.cycle,
        small.cycle
    );
    assert!(big.stride > small.stride.mul(Fx::from_int(5)));
}

#[test]
fn footfalls_come_in_the_animals_order() {
    // A foot lands at `1 - offset` of the cycle.
    let land = |off: Fx| {
        let l = Fx::ONE.sub(off);
        if l >= Fx::ONE { l.sub(Fx::ONE) } else { l }
    };
    let g = cast("Ridgeback");
    let w = locomotion::walking(g, &posture::posture(g));
    assert_eq!(w.kind, GaitKind::Walk);
    // Lateral sequence: hind left, fore left, hind right, fore right.
    let order = [
        land(w.offset[3][0]),
        land(w.offset[0][0]),
        land(w.offset[3][1]),
        land(w.offset[0][1]),
    ];
    assert!(order.windows(2).all(|p| p[0] < p[1]), "{order:?}");
    // A trot moves diagonal pairs together.
    let q = posture::posture(g);
    let trot = locomotion::at(g, &q, Fx::ONE);
    assert_eq!(trot.kind, GaitKind::Trot);
    assert_eq!(trot.offset[0][0], trot.offset[3][1]);
    assert_eq!(trot.offset[0][1], trot.offset[3][0]);
    // A spider's wave runs from the back forward on each side.
    let s = cast("Broodmother");
    let wave = locomotion::walking(s, &posture::posture(s));
    assert_eq!(wave.kind, GaitKind::Wave);
    let side = [
        land(wave.offset[3][0]),
        land(wave.offset[2][0]),
        land(wave.offset[1][0]),
        land(wave.offset[0][0]),
    ];
    assert!(side.windows(2).all(|p| p[0] < p[1]), "{side:?}");
}

#[test]
fn a_planted_foot_stays_where_it_was_put() {
    for (_, name, g, _) in CAST {
        let q = posture::posture(g);
        let p = proportions(g);
        for gait in locomotion::repertoire(g, &q).into_iter().flatten() {
            for i in 0..4 {
                if !gait.walks[i] {
                    continue;
                }
                let standing = p.limb[i];
                // Where the foot is on the floor: its place under the body,
                // plus how far the body has come.
                let world = |phase: Fx| {
                    let f = locomotion::foot_at(&gait, &p, i, 0, phase, Fx::ZERO);
                    (f, f.forward.mul(standing).add(gait.stride.mul(phase)))
                };
                let mut planted: Option<Fx> = None;
                for k in 0..64 {
                    let (f, x) = world(Fx::ratio(k, 64));
                    if f.lift == Fx::ZERO && k > 0 && world(Fx::ratio(k - 1, 64)).0.lift == Fx::ZERO
                    {
                        if let Some(at) = planted {
                            let slide = x.sub(at).abs();
                            assert!(
                                slide <= gait.stride.mul(fx(1, 20)),
                                "{name} {}: foot {i} slides {slide:?} m",
                                gait.kind.name()
                            );
                        } else {
                            planted = Some(x);
                        }
                    } else {
                        planted = None;
                    }
                }
            }
        }
    }
}

#[test]
fn moves_and_gaits_are_the_same_every_time() {
    let g = cast("Mantis");
    let q = posture::posture(g);
    let a = verdict::judge(g, &q);
    let b = verdict::judge(g, &q);
    for (x, y) in a.iter().zip(b.iter()) {
        assert_eq!((x.change, x.beyond, x.swing), (y.change, y.beyond, y.swing));
    }
}
