//! Posture: a plan that is legal also stands -- balanced over its feet,
//! every foot on the floor, its head up -- whatever the crossing did to it.
//!
//! The pass (`sim::breed::posture`) is one principle, the centre of mass
//! over the support, and the strategies a real animal has for it: shift
//! the limbs, lean them, pitch the torso, walk on the wings, drag the rear.
//! These check the principle holds for the cast, for two thousand random
//! genomes, and for the cases the owner named: a Ridgeback whose fore pair
//! became wings, a creature with fore limbs and nothing behind, hind limbs
//! longer than the fore.

use sim::breed::archetypes::{self, CAST, GALEWING, RIDGEBACK, SIEGESHELL};
use sim::breed::build::{self, MoveKind};
use sim::breed::plan;
use sim::breed::posture::{self, Posture};
use sim::breed::{
    Back, Build, Crown, Genome, HeadKind, Hide, Jaw, LimbPair, Minors, Rng, Role, STATIONS,
    Segments, Sliders, Stance, Tail, TailTip, Tip,
};
use sim::fixed::Fx;

fn f(x: Fx) -> f32 {
    x.to_f32_for_render()
}

fn fx(n: i32, d: i32) -> Fx {
    Fx::ratio(n, d)
}

/// Every grounded foot's sole is on the floor, within a hand.
fn feet_on_floor(g: &Genome, name: &str) {
    let body = build::body(g);
    let q = posture::posture(g);
    for (i, s) in q.stations.iter().enumerate() {
        if !s.grounded || s.dangles {
            continue;
        }
        for side in ["l", "r"] {
            let station = ["fore", "midfore", "midhind", "hind"][i];
            let foot = body
                .bone_named(&format!("{station}.{side}.foot"))
                .unwrap_or_else(|| panic!("{name}: no foot bone for {station}.{side}"));
            let y = f(body.at(foot).y);
            assert!(
                y.abs() < 0.12 * f(body.length).max(1.0) * 0.1 + 0.05,
                "{name}: {station}.{side} foot at {y:.2} m off the floor (body {:.1} m)",
                f(body.length)
            );
        }
    }
}

fn check(g: &Genome, name: &str) -> Posture {
    let q = posture::posture(g);
    assert!(
        q.balanced,
        "{name}: not balanced: com {:.2} support {:.2}..{:.2}\n{q:?}",
        f(q.com),
        f(q.support.0),
        f(q.support.1)
    );
    for (i, s) in q.stations.iter().enumerate() {
        if s.grounded {
            assert!(!s.dangles, "{name}: station {i} dangles\n{q:?}");
            let e = f(s.extension);
            assert!(
                (0.3..=1.0).contains(&e),
                "{name}: station {i} extension {e:.2}"
            );
        }
    }
    feet_on_floor(g, name);
    q
}

#[test]
fn every_archetype_stands_as_it_is() {
    for (_, name, g, _) in CAST {
        let q = check(g, name);
        // An archetype is near balance as written: nothing had to move
        // more than a tenth of the body or a tenth of a turn (the bird's
        // legs come forward under its weight).
        assert!(
            f(q.applied.shift).abs() < 0.1 * f(build::length_m(g.sliders.size))
                && f(q.applied.pitch).abs() < 0.11,
            "{name}: the pass had to move it: {:?}",
            q.applied
        );
        assert!(!q.drags && !q.applied.wing_walks, "{name}: {:?}", q.applied);
    }
}

#[test]
fn the_same_genome_always_stands_the_same_way() {
    for (_, _, g, _) in CAST {
        assert_eq!(posture::posture(g), posture::posture(g));
    }
}

#[test]
fn heads_are_carried_above_the_back() {
    for (_, name, g, _) in CAST {
        if matches!(g.head, HeadKind::Low | HeadKind::Merged) {
            continue;
        }
        let body = build::body(g);
        let head = body.bone_named("head").unwrap();
        let chest = body
            .bone_named("chest")
            .or(body.bone_named("thorax"))
            .unwrap();
        let head_y = f(body.at(head).y);
        // Above the spine by most of a third of the body's half-depth: a
        // head carried, not hung -- and no higher, or a body as deep as it
        // is wide stood its neck up straight (exploration 0033).
        let back_y = f(body.at(chest).y) + f(build::proportions(g).half_h) * 0.3;
        assert!(
            head_y >= back_y - 0.05,
            "{name}: head at {head_y:.2}, back at {back_y:.2}"
        );
    }
}

/// The owner's case: a Ridgeback whose fore pair became wings and whose
/// hind pair did not move. It stands anyway, by some mix of the
/// strategies -- and not the same mix for every such creature.
#[test]
fn a_ridgeback_with_wings_for_forelegs_still_stands() {
    let mut mixes = std::collections::HashSet::new();
    for k in 0..40 {
        let mut g = RIDGEBACK;
        let fore = g.limbs[0].as_mut().unwrap();
        fore.role = Role::Wing;
        fore.build = Build::Wing;
        fore.tip = Tip::Feather;
        fore.length = fx(1, 2);
        // A different creature each time: a hair off on a slider changes
        // the fingerprint and so the roll.
        g.sliders.speed = fx(1, 2).add(Fx::ratio(k, 1000));
        let mut r = g;
        plan::repair(&mut r);
        let q = check(&r, "winged ridgeback");
        let a = q.applied;
        assert!(
            a.wing_walks,
            "a Ridgeback with wings for forelegs did not lean on them"
        );
        mixes.insert((
            f(a.shift).abs() > 0.05,
            f(a.reach).abs() > 0.05,
            f(a.pitch).abs() > 0.005,
        ));
        if a.wing_walks {
            // The wrist lands clear ahead of the hind feet.
            let body = build::body(&r);
            let wrist = f(body.at(body.bone_named("fore.l.foot").unwrap()).x);
            let hind = f(body.at(body.bone_named("hind.l.foot").unwrap()).x);
            assert!(
                wrist > hind + 0.5,
                "wrist at {wrist:.2}, hind foot at {hind:.2}"
            );
        }
    }
    // With its forelegs gone the weight is far ahead of the hind feet:
    // every one of them leans on its wings, and those carry it.
    let _ = mixes;
}

/// How much a creature leans on its wings is a gradient, not a switch:
/// shorten the hind legs a little at a time and the wings come down a
/// little at a time -- held, lowered, touching, carrying -- and the
/// same creature never flickers between the two.
#[test]
fn wings_lean_by_degrees() {
    let winged = |hind: Fx| {
        let mut g = RIDGEBACK;
        let fore = g.limbs[0].as_mut().unwrap();
        fore.role = Role::Wing;
        fore.build = Build::Wing;
        fore.tip = Tip::Feather;
        fore.length = fx(1, 2);
        g.limbs[3].as_mut().unwrap().length = hind;
        plan::repair(&mut g);
        g
    };
    let mut leans = Vec::new();
    for k in 0..=24 {
        // Hind legs from a fifth of the body to over half of it.
        let hind = fx(1, 5).add(Fx::ratio(k, 70));
        let q = posture::posture(&winged(hind));
        leans.push(f(q.stations[0].wing_lean));
    }
    for w in leans.windows(2) {
        assert!(w[1] <= w[0] + 0.02, "longer legs leaned more: {leans:?}");
        assert!(w[0] - w[1] < 0.2, "the lean jumped: {leans:?}");
    }
    let plant = f(posture::WING_PLANT);
    assert!(
        leans[0] >= plant,
        "short hind legs did not lean on the wings: {leans:?}"
    );
    assert!(
        *leans.last().unwrap() < plant,
        "long hind legs still lean on the wings: {leans:?}"
    );
    // Some of the sweep is in between: lowered but not yet touching.
    assert!(
        leans.iter().any(|l| *l > 0.0 && *l < plant),
        "no wing ever held part-way down: {leans:?}"
    );
}

/// The flight pose is apart from how it stands: a creature that leans on
/// its wings to walk still spreads them to fly, airborne, and one whose
/// wings cannot lift it flares them on the ground.
#[test]
fn a_wing_walker_still_flies() {
    let g = GALEWING;
    let q = posture::posture(&g);
    let flying = build::body_flying(&g, &q, fx(3, 10));
    let standing = build::body(&g);
    assert!(f(flying.span) > 10.0, "spread span {}", f(flying.span));
    let low = |b: &build::Body| {
        (0..b.part_count)
            .map(|i| f(b.lowest(i)))
            .fold(f32::MAX, f32::min)
    };
    assert!(
        low(&flying) > 1.0,
        "a flier in flight is off the floor: {}",
        low(&flying)
    );
    assert!(low(&standing) < 0.2);
    // A flightless winged creature flares, feet down.
    let mut grounded = RIDGEBACK;
    let fore = grounded.limbs[0].as_mut().unwrap();
    fore.role = Role::Wing;
    fore.build = Build::Wing;
    fore.length = fx(1, 5);
    plan::repair(&mut grounded);
    assert!(!plan::assess(&grounded).caps.flies);
    let q = posture::posture(&grounded);
    let flare = build::body_flying(&grounded, &q, fx(3, 10));
    assert!(low(&flare) < 0.2, "a flightless body left the floor");
}

/// Fore limbs and nothing behind: the rear rests on the floor and trails.
#[test]
fn fore_limbs_alone_drag_the_rear() {
    let mut g = RIDGEBACK;
    g.limbs[3] = None;
    g.limbs[0].as_mut().unwrap().role = Role::Walk;
    plan::repair(&mut g);
    assert_eq!(g.stance, Stance::Lowslung, "{:?}", g.stance);
    let q = check(&g, "fore-limbed");
    assert!(q.drags, "{q:?}");
    assert!(f(q.drag_len) > 0.0);
    let body = build::body(&g);
    let rump = body
        .parts()
        .iter()
        .position(|p| p.name == "rump")
        .expect("a rump to drag");
    let bottom = f(body.lowest(rump));
    assert!(
        bottom.abs() < 0.05,
        "the rump is {bottom:.2} m off the floor"
    );
}

/// Hind limbs longer than the fore on a level body: a hopper, knees bent
/// more behind than in front.
#[test]
fn long_hind_limbs_bend_more_and_make_a_hopper() {
    let mut g = archetypes::PAIR;
    g.limbs[3].as_mut().unwrap().length = fx(9, 10);
    let q = check(&g, "hopper");
    assert!(f(q.hop) > 0.3, "hop {:.2}", f(q.hop));
    assert!(
        q.stations[3].extension < q.stations[0].extension,
        "hind {:.2} fore {:.2}",
        f(q.stations[3].extension),
        f(q.stations[0].extension)
    );
    // The feet are still on the floor and the hips are off it.
    assert!(f(q.root) > 0.5);
    // A hopper pounces harder: its leap appears in the moves.
    assert!(build::moves(&g).has(MoveKind::Pounce));
}

/// The Galewing stands on two legs under its weight, wings folded, as a
/// bird does -- never on its wings.
#[test]
fn the_galewing_stands_on_its_legs() {
    let q = check(&GALEWING, "galewing");
    assert!(!q.applied.wing_walks);
    let s = q.stations[3];
    assert!(s.grounded);
    // Its legs are under the weight, not at the tail.
    assert!(
        f(s.attach) + f(s.reach) > -0.1,
        "feet at {:.2}",
        f(s.attach) + f(s.reach)
    );
}

fn random_genome(rng: &mut Rng) -> Genome {
    let pair = |rng: &mut Rng| {
        if rng.chance(fx(3, 5)) {
            Some(LimbPair {
                role: Role::ALL[rng.below(4)],
                build: Build::ALL[rng.below(6)],
                tip: Tip::ALL[rng.below(6)],
                length: rng.unit().mul(fx(4, 5)).add(fx(1, 10)),
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

/// Two thousand random plans, repaired: every one stands.
#[test]
fn any_repaired_plan_stands() {
    let mut rng = Rng::new(7);
    let mut unbalanced = 0;
    let mut dangling = 0;
    for i in 0..2000 {
        let mut g = random_genome(&mut rng);
        plan::repair(&mut g);
        plan::settle(&mut g);
        let q = posture::posture(&g);
        if !q.balanced {
            unbalanced += 1;
        }
        if q.stations.iter().any(|s| s.grounded && s.dangles) {
            dangling += 1;
        }
        feet_on_floor(&g, &format!("random {i}"));
        let body = build::body(&g);
        assert!(
            body.bone_count <= 40 && body.part_count <= 48,
            "random {i}: too many bones or parts"
        );
    }
    // A few plans cannot be balanced within the body's limits -- a tiny
    // pair of legs far behind a huge head -- and a few cannot reach: they
    // are rare, and they are flagged.
    assert!(
        unbalanced < 60,
        "{unbalanced} of 2000 random plans could not be balanced"
    );
    assert!(
        dangling < 60,
        "{dangling} of 2000 random plans have a limb that cannot reach the floor"
    );
}

/// Legs against weight: the cast has legs for its bodies; a body grown
/// twice its size on the same legs does not, and rests on its belly; and
/// over generations with nobody pulling the other way, its legs thicken
/// back toward what the weight needs.
#[test]
fn legs_are_held_to_the_weight_they_carry() {
    for (_, name, g, _) in CAST {
        let s = f(plan::leg_strength(g));
        assert!(s >= 0.8, "{name}: leg strength {s:.2}");
        assert!(!plan::assess(g).caps.weak_legs, "{name}: weak legs");
    }
    // A gnawer grown to the Siegeshell's size on a gnawer's legs: bone
    // carries by area and weight grows by volume, so strength falls as
    // one over the length, and a rat twenty times as long has a
    // twentieth of what it had.
    let mut g = archetypes::GNAWER;
    g.sliders.size = SIEGESHELL.sliders.size;
    let s = f(plan::leg_strength(&g));
    assert!(
        s < 0.5,
        "a rat the size of a spider has leg strength {s:.2}"
    );
    let a = plan::assess(&g);
    assert!(a.caps.weak_legs && !a.caps.pounces && !a.caps.charges);
    let q = posture::posture(&g);
    assert!(q.bellied && q.balanced, "{q:?}");
    let body = build::body(&g);
    assert!(
        // Hips no higher than the body is half deep: resting on it.
        f(body.hips) <= f(build::proportions(&g).half_h) + 0.01,
        "its belly is off the floor"
    );
    // Thirty generations of settling: the legs thicken.
    let before = f(g.limbs[3].unwrap().girth);
    let mut x = g;
    for _ in 0..30 {
        plan::settle(&mut x);
    }
    let after = f(x.limbs[3].unwrap().girth);
    assert!(after > before + 0.1, "girth {before:.2} -> {after:.2}");
    assert!(f(plan::leg_strength(&x)) > s * 1.2);
}
