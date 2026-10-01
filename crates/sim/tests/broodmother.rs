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

// ---------------------------------------------------------------------------
// §2 · Her moves
// ---------------------------------------------------------------------------

#[test]
fn a_screech_is_always_followed_by_a_slam() {
    // Declared in the move table (`MoveDecl::then`), so it is a fact about
    // the move: the frame the scream's recovery ends, the slam winds up --
    // no pause to think, no choice. Over a long hunt, every time.
    let mut w = hunt(Class::Champion);
    let mut screeches = 0;
    for _ in 0..7200 {
        let before = w.monsters[0].expect("her").doing;
        idle(&mut w);
        let after = w.monsters[0].expect("her").doing;
        if let Doing::Recovery {
            kind: bm::SCREECH,
            left: 0,
        } = before
        {
            screeches += 1;
            assert!(
                matches!(after, Doing::Startup { kind: bm::SLAM, .. }),
                "a screech ended in {after:?}, not the slam"
            );
        }
    }
    assert!(screeches > 0, "she never screeched");
    assert!(bm::MOVES[bm::SCREECH as usize].then == Some(bm::SLAM));
}

#[test]
fn the_stabbing_foot_lands_on_the_disc_the_hit_is_tested_on() {
    // The overlay's rule, for a leg: the foot you watch come down is the disc
    // the telegraph draws and the hit test uses.
    let mut m = Monster::new(SpeciesId::BROODMOTHER);
    let a = bm::SPECIES.attack(bm::STAB);
    for leg in 0..bm::LEG_COUNT {
        let rig = m.rig();
        let home = bm::legs::foot(&m, &rig, leg);
        let disc = V3::new(
            home.x.mul(Fx::ratio(3, 5)),
            Fx::ZERO,
            home.z.mul(Fx::ratio(3, 5)),
        );
        m.aim_at(disc);
        m.own[fight::body::STAB] = leg as i32 + 1;
        m.doing = Doing::Active {
            kind: bm::STAB,
            left: a.active,
        };
        let rig = m.rig();
        let foot = bm::legs::foot(&m, &rig, leg);
        let t = m.telegraph().expect("a stab is drawn");
        let gap = sim::math::wide_flat_dist(foot, t.anchor);
        assert!(
            gap.raw() < t.radius.raw() && foot.y.raw() < Fx::ratio(1, 4).raw(),
            "leg {leg}'s foot is {gap:?} off its disc, {:?} up",
            foot.y
        );
    }
}

#[test]
fn a_broken_leg_never_stabs() {
    let mut m = Monster::new(SpeciesId::BROODMOTHER);
    for leg in 0..bm::LEG_COUNT {
        if bm::middle_leg(leg) {
            let slot = bm::SPECIES
                .break_slot(bm::shin_part(leg))
                .expect("breakable");
            m.breaks[slot] = 0;
        }
    }
    let rig = m.rig();
    for leg in 0..bm::LEG_COUNT {
        let at = bm::legs::foot(&m, &rig, leg);
        let chosen = bm::mind::stab_leg(&m, at).expect("a sound leg");
        assert!(
            !bm::middle_leg(chosen),
            "a broken leg {chosen} was given a stab"
        );
    }
}

// ---------------------------------------------------------------------------
// §2 · The web
// ---------------------------------------------------------------------------

#[test]
fn a_web_root_outlasts_the_brood_arriving() {
    // §2's arithmetic: ninety frames of root is enough for a broodling eight
    // metres away to arrive and bite once -- not twice.
    let sp = &bm::SPECIES;
    let root = sim::hazard::stat(sp, fight::GLOB, sim::hazard::HazardField::Life);
    let run = sim::critter::stat_fx(sp, 0, sim::critter::CritterField::Run);
    let bite = sp.attack(bm::DART);
    let reach = Fx::from_int(8).sub(Knob::RecallRadius.fx().min(bite.hit_x.add(bite.hit_radius)));
    let arrive = reach
        .div(run)
        .mul(Fx::from_int(sim::TICK_HZ as i32))
        .to_int();
    let first = arrive + bite.startup as i32;
    let second = first + bite.active as i32 + bite.recovery as i32 + bite.startup as i32;
    assert!(
        first <= root,
        "a broodling 8 m out lands its first bite at {first}, after the root's {root}"
    );
    assert!(
        second > root,
        "a broodling 8 m out bites twice ({second}) inside the root's {root}"
    );
}

#[test]
fn every_attack_thrown_cuts_a_webbed_fighter_free_sooner() {
    let mut w = hunt(Class::Champion);
    w.players[0].pos = at(-10, 0);
    // Webbed on the floor: a glob under the feet.
    fight::landed(&mut w, 0, 0, bm::WEB_SHOT, false);
    for _ in 0..2 {
        hold_still(&mut w, 0);
    }
    let glob = || sim::hazard::all(&w.lore).find(|(_, h)| h.index() == fight::GLOB);
    assert!(glob().is_some(), "a webbed fighter has no glob under them");
    // Swinging: each attack takes `CutFree` off the root.
    let life = sim::hazard::stat(&bm::SPECIES, fight::GLOB, sim::hazard::HazardField::Life);
    let mut frames = 0;
    while sim::hazard::all(&w.lore).any(|(_, h)| h.index() == fight::GLOB) && frames < life {
        hold_still(&mut w, Input::LEFT);
        frames += 1;
    }
    assert!(
        frames < life,
        "swinging did not cut the root short ({frames} of {life})"
    );
}

/// A frame with her held out of it, fighter zero pressing `bits`.
fn hold_still(w: &mut World, bits: u16) {
    if let Some(m) = w.monsters[0].as_mut() {
        m.pos = at(8, 0);
        m.brain.grace = u16::MAX;
        m.brain.think_left = u16::MAX;
        m.doing = Doing::Prowl;
    }
    for c in w.critters.iter_mut() {
        *c = sim::critter::Critter::EMPTY;
    }
    w.players[1].pos = at(-13, 10);
    w.advance([Input::aimed(bits, 0), Input::default()]);
    for p in w.players.iter_mut() {
        p.health = p.full_health();
    }
}

#[test]
fn a_strand_trips_a_run_and_not_a_crouch() {
    // §2: crossing a strand faster than a crouch walk trips you -- 30 and a
    // stagger -- and a crouch walk steps over it.
    let cross = |bits: u16| {
        let mut w = hunt(Class::Champion);
        w.players[0].pos = at(-8, -3);
        let strand = sim::hazard::Hazard::strand(
            fight::STRAND,
            at(-12, 0),
            at(-4, 0),
            Knob::StrandHalf.fx(),
        );
        sim::hazard::place(&mut w.lore, strand).expect("room");
        let health = w.players[0].health;
        let mut tripped = false;
        for _ in 0..90 {
            hold_still(&mut w, bits);
            w.players[0].health = w.players[0].health.min(health);
            tripped |= matches!(w.players[0].action, sim::state::Action::HitStun { .. });
        }
        (tripped, w.players[0].pos.z)
    };
    // Looking along +x, the strand runs along x at z = 0: walk across it to
    // the right, along +z, with D.
    let (ran, z) = cross(Input::D);
    assert!(z.raw() > 0, "the run never crossed the strand");
    assert!(ran, "a run across a strand did not trip");
    let (crept, z) = cross(Input::D | Input::CROUCH);
    assert!(z.raw() > 0, "the crouch walk never crossed the strand");
    assert!(!crept, "a crouch walk across a strand tripped");
}

#[test]
fn a_pillar_between_her_spinnerets_and_you_stops_the_glob() {
    // §2: the answer to the web shot is a solid between you and the
    // spinnerets. The glob lands at the pillar, and the disc drawn for it is
    // that landing.
    let mut w = hunt(Class::Champion);
    let pillar = w
        .arena()
        .solids
        .iter()
        .find(|s| {
            let thin = |a: Fx, b: Fx| b.sub(a).raw() <= Fx::from_int(3).raw();
            let inside = |a: Fx, b: Fx| a.add(b).abs().raw() < Fx::from_int(20).raw();
            s.min.y.raw() == 0
                && thin(s.min.x, s.max.x)
                && thin(s.min.z, s.max.z)
                && inside(s.min.x, s.max.x)
                && inside(s.min.z, s.max.z)
        })
        .copied()
        .expect("the Hollows has pillars");
    let middle = V3::new(
        pillar.min.x.add(pillar.max.x).mul(Fx::ratio(1, 2)),
        Fx::ZERO,
        pillar.min.z.add(pillar.max.z).mul(Fx::ratio(1, 2)),
    );
    let m = w.monsters[0].as_mut().expect("her");
    // Her back to the pillar, her spinnerets a couple of metres off it; the
    // target behind it.
    m.pos = middle.add(at(10, 0));
    m.yaw = Fx::ZERO;
    m.aim_at(middle.sub(at(3, 0)));
    let m = *m;
    let lands = fight::glob_lands(&w, &m);
    assert!(
        lands.x.raw() > pillar.max.x.raw(),
        "the glob went through the pillar to {lands:?}"
    );
}

// ---------------------------------------------------------------------------
// §6 · Reading it
// ---------------------------------------------------------------------------

#[test]
fn what_is_drawn_under_a_red_sac_is_where_its_brood_land() {
    // The ring under a red sac is the landing the burst spawns at: drawn
    // from the same function, so every broodling born lands inside the ring
    // drawn for it on the frame before.
    // Past the first frame, whose brood are the ones she starts with.
    let mut w = untouched(2);
    let mut checked = 0;
    for _ in 0..3600 {
        let rings: Vec<sim::sign::Sign> = w
            .signs()
            .iter()
            .filter(|s| s.shape == sim::sign::Shape::Ring && s.says == sim::sign::Says::Coming)
            .copied()
            .collect();
        let before = w.critters;
        idle(&mut w);
        for (b, a) in before.iter().zip(w.critters.iter()) {
            let born = a.alive() && (!b.present() || !b.alive());
            if !born {
                continue;
            }
            checked += 1;
            let inside = rings.iter().any(|r| {
                sim::math::wide_flat_dist(r.at, a.pos).raw()
                    <= r.width
                        .mul(Fx::ratio(1, 2))
                        .add(sim::tuning::body_radius())
                        .raw()
            });
            assert!(
                inside,
                "a broodling was born at {:?}, outside every ring drawn",
                a.pos
            );
        }
    }
    assert!(checked >= 4, "only {checked} broodlings were born to check");
}

// ---------------------------------------------------------------------------
// §4 · What you break, and her arc
// ---------------------------------------------------------------------------

fn break_shin(m: &mut Monster, leg: usize) {
    let slot = bm::SPECIES
        .break_slot(bm::shin_part(leg))
        .expect("a middle shin");
    m.breaks[slot] = 0;
}

#[test]
fn two_broken_legs_on_a_side_lower_that_sides_sacs_for_good() {
    let mut m = Monster::new(SpeciesId::BROODMOTHER);
    let standing: Vec<Fx> = (0..bm::SAC_COUNT)
        .map(|i| fight::sac_middle(&m, i).y)
        .collect();
    break_shin(&mut m, 3);
    assert_eq!(bm::legs::list(&m), 0, "one broken leg listed her");
    break_shin(&mut m, 5);
    assert_eq!(
        bm::legs::list(&m),
        1,
        "two broken right legs did not list her right"
    );
    // Every state she can be in: that side's sacs sit lower than the other's,
    // and lower than they stood.
    for doing in [Doing::Prowl, crashed()] {
        m.doing = doing;
        for pair in 0..3 {
            let (l, r) = (
                fight::sac_middle(&m, 2 * pair).y,
                fight::sac_middle(&m, 2 * pair + 1).y,
            );
            assert!(
                r.raw() < l.raw(),
                "pair {pair}: the listed side is not the lower ({doing:?})"
            );
            if doing == Doing::Prowl {
                assert!(
                    r.raw() < standing[2 * pair + 1].raw(),
                    "pair {pair} did not come down"
                );
            }
        }
    }
    // And the listed fore sac is a hop for the whole roster, the Bulwark's
    // included: the ground game is his way to the sacs outside the slam.
    fn listed(m: &mut Monster) {
        fight::only_sac(m, 1);
        for leg in [3, 5] {
            let slot = bm::SPECIES
                .break_slot(bm::shin_part(leg))
                .expect("breakable");
            m.breaks[slot] = 0;
        }
    }
    for class in [Class::Bulwark, Class::Champion] {
        let r = reachcheck::reach_with(
            SpeciesId::BROODMOTHER,
            Doing::Prowl,
            class,
            bm::sac_part(1),
            listed,
        );
        assert!(r.hop, "{class:?} cannot hop to the listed fore sac");
    }
}

#[test]
fn popping_the_last_sac_collapses_her_and_she_never_spawns_again() {
    let mut w = untouched(1);
    for i in 0..bm::SAC_COUNT {
        let m = w.monsters[0].as_mut().expect("her");
        m.take_hit(bm::sac_part(i), Knob::SacHealth.raw() * 4);
        idle(&mut w);
    }
    let m = w.monsters[0].expect("her");
    assert!(fight::enraged(&m), "six sacs gone and she is not enraged");
    assert!(
        matches!(m.doing, Doing::Toppled { .. }),
        "the last pop did not collapse her: {:?}",
        m.doing
    );
    let brood = |w: &World| fight::brood(w);
    let broken = w.pack.map(|p| p.mood) == Some(sim::pack::mood::BROKEN);
    assert!(broken, "the brood did not break at her collapse");
    for _ in 0..(Knob::SacRipen.raw() * 2) {
        idle(&mut w);
        assert!(
            (0..bm::SAC_COUNT).all(|i| fight::site(&w, i).0 == fight::sac::SCARRED),
            "a site laid again after the collapse"
        );
    }
    assert_eq!(
        brood(&w),
        0,
        "the brood is still in the fight after her collapse"
    );
}

#[test]
fn below_the_clutch_every_living_sac_goes_red_together() {
    let mut w = untouched(1);
    let m = w.monsters[0].as_mut().expect("her");
    m.take_hit(bm::sac_part(5), Knob::SacHealth.raw() * 4);
    let full = m.sp().health();
    m.health = Fx::from_int(full).mul(Knob::ClutchHealth.fx()).to_int() - 1;
    for _ in 0..Knob::ClutchFrames.raw() + 2 {
        idle(&mut w);
    }
    for i in 0..bm::SAC_COUNT - 1 {
        let (state, _) = fight::site(&w, i);
        assert!(
            state != fight::sac::SWELLING || fight::red(&w, i),
            "sac {i} is not red {} frames into the clutch",
            Knob::ClutchFrames.raw()
        );
    }
}

#[test]
fn a_struck_sac_turns_the_brood_near_her_on_the_one_who_struck_it() {
    let mut w = hunt(Class::Champion);
    for _ in 0..30 {
        idle(&mut w);
    }
    // Fighter one beside her, fighter zero far off and nearer every
    // broodling; a sac struck names fighter one.
    let m = w.monsters[0].expect("her");
    let beside = fight::sac_middle(&m, 0);
    w.players[1].pos = V3::new(beside.x, Fx::ZERO, beside.z.add(Fx::from_int(3)));
    w.players[0].pos = at(-13, -10);
    w.monsters[0]
        .as_mut()
        .expect("her")
        .take_hit(bm::sac_part(0), 1);
    for _ in 0..3 {
        w.advance([Input::default(); MAX_PLAYERS]);
    }
    let m = w.monsters[0].expect("her");
    assert_eq!(
        fight::guarded(&m),
        Some(1),
        "the guard did not name the one beside the sac"
    );
    let near: Vec<_> = w
        .critters
        .iter()
        .filter(|c| {
            c.alive()
                && sim::math::wide_flat_dist(c.pos, m.pos).raw() <= Knob::GuardRadius.fx().raw()
        })
        .collect();
    assert!(!near.is_empty(), "no broodling near her to check");
    assert!(
        near.iter().all(|c| c.target == 1),
        "a broodling near her is not on the attacker"
    );
}
