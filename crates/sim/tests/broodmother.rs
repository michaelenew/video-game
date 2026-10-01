//! The Broodmother's rules: her body against the roster's reach, the sac
//! clock and the brood it feeds, her moves and the chain, the web, her legs
//! and her arc. Sentences that state the rule, as
//! `docs/design/creatures/broodmother.md` §10 lists them; a failure here is a
//! bug or a design decision that changed without the document changing with
//! it.

use sim::class::ALL_CLASSES;
use sim::fixed::Fx;
use sim::monster::{Doing, Monster};
use sim::reachcheck::{self, Reach};
use sim::species::SpeciesId;
use sim::species::broodmother::{self as bm, Knob, fight};
use sim::state::MAX_PLAYERS;
use sim::{Class, Input, V3, World};

fn hunt(class: Class) -> World {
    World::hunt_of([class; MAX_PLAYERS], SpeciesId::BROODMOTHER)
}

fn at(x: i32, z: i32) -> V3 {
    V3::new(Fx::from_int(x), Fx::ZERO, Fx::from_int(z))
}

/// Only sac `i` on her back, for the reach instrument: a swing that touches
/// two is asked about the one being measured.
const ONLY: [fn(&mut Monster); bm::SAC_COUNT] = [
    |m| fight::only_sac(m, 0),
    |m| fight::only_sac(m, 1),
    |m| fight::only_sac(m, 2),
    |m| fight::only_sac(m, 3),
    |m| fight::only_sac(m, 4),
    |m| fight::only_sac(m, 5),
];

fn reach(doing: Doing, class: Class, site: usize) -> Reach {
    reachcheck::reach_with(
        SpeciesId::BROODMOTHER,
        doing,
        class,
        bm::sac_part(site),
        ONLY[site],
    )
}

/// The slam's recovery, half a second in: down on the floor, before the
/// lift.
fn crashed() -> Doing {
    let a = bm::SPECIES.attack(bm::SLAM);
    Doing::Recovery {
        kind: bm::SLAM,
        left: a.recovery.saturating_sub(30),
    }
}

// ---------------------------------------------------------------------------
// §1 · The body against the roster's reach
// ---------------------------------------------------------------------------

#[test]
fn no_sac_is_inside_a_hop_and_a_swing_for_the_bulwark_or_the_champion_while_she_stands() {
    // The fight's premise, written as a rule: standing, the two lowest
    // hoppers cannot pop a sac from beside her, at the top of a hop or on
    // the floor, with either auto and the crosshair on it. (From *under* her
    // the Champion's spear, thrust straight up, reaches the fore and mid
    // pairs through her: that is the tempting spot §3 covers with the bare
    // slam, and it is reported, not forbidden.)
    for class in [Class::Bulwark, Class::Champion] {
        for site in 0..bm::SAC_COUNT {
            let r = reach(Doing::Prowl, class, site);
            assert!(
                !r.hop && !r.standing,
                "{class:?} reaches the {} standing beside her ({r:?})",
                bm::PARTS[bm::sac_part(site)].name
            );
        }
    }
}

#[test]
fn the_fore_pair_is_a_hop_for_the_highest_hoppers_and_the_aft_pair_is_the_floats() {
    // §1, rules 2 and 3: the fore pair is a sixty-frame commitment in the
    // air for the classes that hop five metres -- the Reaver and the Dual
    // mage -- and the aft pair is for the Dual mage's float (and anybody's
    // ranged shot, `a_skillshot_on_the_crosshair_meets_the_sac_under_it`).
    for class in [Class::ShadowReaver, Class::DualMage] {
        for site in [0, 1] {
            assert!(
                reach(Doing::Prowl, class, site).hop,
                "{class:?} cannot hop to the fore sac {site}"
            );
        }
    }
    for site in [4, 5] {
        assert!(
            reach(Doing::Prowl, Class::DualMage, site).hop,
            "the Dual mage's float does not reach the aft sac {site}"
        );
    }
}

#[test]
fn the_slam_brings_every_living_sac_inside_every_classs_reach() {
    // §1, rule 4, the window: through the slam's recovery every living sac
    // is a standing swing away for every class -- from beside the abdomen
    // where it lies, or, for a caster, from wherever she stands with the
    // crosshair on it.
    for class in ALL_CLASSES {
        for site in 0..bm::SAC_COUNT {
            let r = reach(crashed(), class, site);
            let ranged = class == Class::Elementalist && skillshot_lands(crashed(), site, 9, 0);
            assert!(
                r.standing || ranged,
                "{class:?} cannot reach the {} standing in the slam ({r:?})",
                bm::PARTS[bm::sac_part(site)].name
            );
        }
    }
}

/// An Elementalist `metres` from a sac's floor point, on `bearing`, standing,
/// with the crosshair on the sac's middle: does either auto pop some of it?
fn skillshot_lands(doing: Doing, site: usize, metres: i32, bearing: i32) -> bool {
    let mut base = reachcheck::stage(SpeciesId::BROODMOTHER, Class::Elementalist);
    let centre = reachcheck::centre(&base);
    reachcheck::pin_with(&mut base, doing, centre, ONLY[site]);
    let part = bm::sac_part(site);
    let target = reachcheck::middle(&base, part);
    let dir = V3::from_turns(Fx::from_raw(bearing));
    let from = V3::new(target.x, Fx::ZERO, target.z).add(dir.scale(Fx::from_int(metres)));
    let slot = bm::SPECIES.break_slot(part).expect("a sac is breakable");
    [Input::LEFT, Input::RIGHT].into_iter().any(|button| {
        let mut w = base.clone();
        w.players[0].pos = from;
        let before = w.monsters[0].expect("her").breaks[slot];
        (0..90).any(|frame| {
            reachcheck::pin_with(&mut w, doing, centre, ONLY[site]);
            let p = &w.players[0];
            let d = target.sub(p.pos);
            let aim = (sim::math::atan2_turns(d.z, d.x).raw() & 0xFFFF) as u16;
            let pitch = sim::aim::look_onto_closely(p.pos, aim, p.aloft, target);
            let bits = if frame == 3 { button } else { 0 };
            w.advance([Input::looking_at(bits, aim, pitch), Input::default()]);
            w.monsters[0].expect("her").breaks[slot] < before
        })
    })
}

#[test]
fn a_skillshot_on_the_crosshair_meets_the_sac_under_it() {
    // §12's first question, decided by a test: bodies are not on the
    // crosshair's ray, so the ray passes through a sac to whatever is behind
    // it -- and the straight line from the caster to that point still meets
    // the sac, at every range the cave allows, from every side, standing and
    // in the slam. Nothing in `aim.rs` changed for her.
    for doing in [Doing::Prowl, crashed()] {
        for site in 0..bm::SAC_COUNT {
            for metres in [6, 9, 12] {
                for bearing in [0, 1 << 13, 1 << 14, 3 << 13, 1 << 15, 3 << 14] {
                    assert!(
                        skillshot_lands(doing, site, metres, bearing),
                        "a shot with the crosshair on the {} from {metres} m (bearing {bearing}) \
                         missed it ({doing:?})",
                        bm::PARTS[bm::sac_part(site)].name
                    );
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// §1 · The sac clock
// ---------------------------------------------------------------------------

/// A hunt in which nobody fights: the hunters out of her way and unhurt.
fn untouched(frames: u32) -> World {
    let mut w = hunt(Class::Bulwark);
    for _ in 0..frames {
        idle(&mut w);
    }
    w
}

/// One frame of nobody fighting: the hunters kept whole and far away.
fn idle(w: &mut World) {
    w.players[0].pos = at(-13, -10);
    w.players[1].pos = at(-13, 10);
    w.advance([Input::default(); MAX_PLAYERS]);
    for p in w.players.iter_mut() {
        p.health = p.full_health();
    }
}

#[test]
fn the_brood_never_exceed_the_cap_and_a_ripe_sac_holds_while_it_is_full() {
    // A hunt nobody touches ends, as §10 M2 says, with the brood at its cap
    // and every sac held: red, pulsing, waiting for room.
    let mut w = hunt(Class::Bulwark);
    let cap = Knob::BroodCap.raw() as usize;
    for _ in 0..5400 {
        idle(&mut w);
        assert!(fight::brood(&w) <= cap, "{} brood alive", fight::brood(&w));
    }
    assert_eq!(fight::brood(&w), cap, "the brood never filled");
    for i in 0..bm::SAC_COUNT {
        assert_eq!(
            fight::site(&w, i).0,
            fight::sac::HELD,
            "sac {i} is not held with the brood full"
        );
        assert!(fight::red(&w, i), "a held sac is not red");
    }
    // Room again: a held sac bursts at once.
    let kill = (0..sim::critter::MAX_CRITTERS)
        .filter(|i| w.critters[*i].alive())
        .take(2)
        .collect::<Vec<_>>();
    for i in kill {
        let pack = w.pack.as_mut().expect("her brood");
        sim::pack::killed(pack, &mut w.critters, i);
        w.critters[i].timer = 0;
    }
    idle(&mut w);
    assert_eq!(fight::brood(&w), cap, "a held sac did not burst into room");
}

#[test]
fn a_burst_sac_grows_back_and_a_popped_one_never_does() {
    let mut w = untouched(1);
    // Sac 0 starts nearest red: watch it burst, empty, and lay again.
    let mut burst_at = None;
    let mut laid_at = None;
    for f in 0..3000 {
        idle(&mut w);
        let (state, _) = fight::site(&w, 0);
        if state == fight::sac::EMPTY && burst_at.is_none() {
            burst_at = Some(f);
        }
        if burst_at.is_some() && state == fight::sac::SWELLING && laid_at.is_none() {
            laid_at = Some(f);
        }
    }
    let (burst, laid) = (burst_at.expect("it burst"), laid_at.expect("it laid again"));
    assert_eq!(
        laid - burst,
        Knob::SacRegrow.raw(),
        "it did not lay again on its clock"
    );
    // Pop one: a scar, for good -- and her own health and strain pay for it.
    let mut w = untouched(1);
    let m = w.monsters[0].as_mut().expect("her");
    let before = (m.health, m.strain);
    let sac = bm::sac_part(3);
    let dealt = m.take_hit(sac, Knob::SacHealth.raw() * 4);
    assert!(dealt > 0);
    assert_eq!(
        m.health,
        before.0 - Knob::PopDamage.raw(),
        "a pop did not deal her its damage"
    );
    assert!(
        m.strain >= before.1 + Knob::PopStrain.raw(),
        "a pop added no strain"
    );
    for _ in 0..(Knob::SacRipen.raw() + Knob::SacRegrow.raw()) * 2 {
        idle(&mut w);
        assert_eq!(
            fight::site(&w, 3).0,
            fight::sac::SCARRED,
            "a popped sac came back"
        );
    }
}

#[test]
fn a_hit_on_a_sac_goes_to_the_sac_and_not_to_her() {
    let mut w = untouched(1);
    let m = w.monsters[0].as_mut().expect("her");
    let health = m.health;
    let slot = bm::SPECIES.break_slot(bm::sac_part(0)).expect("breakable");
    let sac = m.breaks[slot];
    m.take_hit(bm::sac_part(0), 40);
    assert_eq!(m.health, health, "a hit on a sac hurt her");
    assert!(m.breaks[slot] < sac, "a hit on a sac did not hurt the sac");
}

#[test]
fn a_scarred_or_empty_site_has_nothing_to_hit() {
    let mut w = untouched(1);
    let m = w.monsters[0].as_mut().expect("her");
    m.take_hit(bm::sac_part(2), Knob::SacHealth.raw() * 4);
    let rig = m.rig();
    assert!(!rig.there(bm::sac_part(2)), "a popped sac still has a body");
    assert!(rig.there(bm::sac_part(1)), "a living sac has no body");
}
