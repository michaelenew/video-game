//! Perception (bestiary P5) and the view question (A5), stood up on the dev
//! creature, the sentinel (`sim::species::sentinel`): it sees inside a cone,
//! not through a solid or a cloud, not into a scarred side, and hears the
//! noise ring when it sees nobody. Also here: a creature that collides with
//! the arena's solids, and the hook that tells it so.
//!
//! What the Ridgeback perceives -- everybody -- is pinned by the pinned hunts,
//! which hash every glance it takes.

use sim::aim::{self, Scene};
use sim::arena::{ArenaId, Terrain};
use sim::fixed::Fx;
use sim::hazard::{self, Hazard};
use sim::monster::{Doing, Monster, Quarry, Senses};
use sim::noise::{self, NoiseKind};
use sim::species::{SpeciesId, ridgeback, sentinel};
use sim::state::MAX_PLAYERS;
use sim::{Class, Input, V3, World};

fn at(x: i32, z: i32) -> V3 {
    V3::new(Fx::from_int(x), Fx::ZERO, Fx::from_int(z))
}

/// Somewhere the sample could never have come from.
fn nowhere() -> V3 {
    at(0, -13)
}

/// A sentinel hunt with an empty floor, the creature at `beast` facing +x,
/// fighter zero at `who` and fighter one far behind it, its next glance due.
fn watching(place: ArenaId, beast: V3, who: V3) -> World {
    let mut w = World::hunt_in(
        [Class::Champion; MAX_PLAYERS],
        [Some(SpeciesId::SENTINEL), None],
        place,
    );
    w.lore.set_word(sentinel::word::LAID, 1);
    let m = w.monsters[0].as_mut().unwrap();
    m.pos = beast;
    m.yaw = Fx::ZERO;
    m.brain.seen = nowhere();
    m.brain.glance_left = 0;
    m.brain.target = 1;
    w.players[0].pos = who;
    w.players[1].pos = beast.sub(at(12, 0));
    w.players[1].health = 0;
    w
}

/// One frame, nobody pressing anything. The glance happens on it.
fn glance(w: &mut World) -> V3 {
    w.advance([Input::default(); MAX_PLAYERS]);
    w.monsters[0].unwrap().brain.seen
}

#[test]
fn it_samples_a_fighter_inside_its_cone_and_not_one_behind_it() {
    let mut ahead = watching(ArenaId::PROVING_GROUND, at(0, 0), at(10, 2));
    assert_eq!(glance(&mut ahead), ahead.players[0].pos);

    let mut behind = watching(ArenaId::PROVING_GROUND, at(0, 0), at(-10, 2));
    assert_eq!(
        glance(&mut behind),
        nowhere(),
        "behind it, nothing is sampled"
    );
}

#[test]
fn it_does_not_see_through_a_solid() {
    // The range's tower stands between x 17 and 23; the creature's head is
    // some metres ahead of where it stands.
    let mut hidden = watching(ArenaId::RANGE, at(0, 0), at(30, 0));
    assert_eq!(glance(&mut hidden), nowhere(), "the tower is in the way");
    let mut clear = watching(ArenaId::RANGE, at(0, 0), at(30, -20));
    assert_eq!(glance(&mut clear), clear.players[0].pos);
}

#[test]
fn it_does_not_see_through_smoke() {
    let mut w = watching(ArenaId::PROVING_GROUND, at(-10, 10), at(2, 10));
    hazard::place(
        &mut w.lore,
        Hazard::disc(sentinel::SMOKE, at(-4, 10), Fx::from_int(2)),
    );
    assert_eq!(glance(&mut w), nowhere());
}

#[test]
fn a_scarred_side_is_blind() {
    // Sixty degrees to its right: inside a seventy-two degree cone, and inside
    // a blind arc centred square off the right.
    let right = V3::new(Fx::from_int(5), Fx::ZERO, Fx::ratio(866, 100));
    for (scar, sees) in [(0, true), (1, false), (-1, true)] {
        let mut w = watching(ArenaId::PROVING_GROUND, at(0, 0), right);
        w.lore.set_int(sentinel::word::SCAR, scar);
        let seen = glance(&mut w);
        assert_eq!(seen == w.players[0].pos, sees, "scarred {scar}");
    }
}

#[test]
fn seeing_nobody_it_takes_its_sample_from_what_it_heard() {
    // Walking behind it: unseen, and loud.
    let mut w = watching(ArenaId::PROVING_GROUND, at(4, 10), at(1, 10));
    let mut heard = None;
    for _ in 0..60 {
        if let Some(m) = w.monsters[0].as_mut() {
            m.brain.grace = u16::MAX;
            m.pos = at(4, 10);
            m.yaw = Fx::ZERO;
        }
        w.advance([Input::aimed(Input::S, 0), Input::default()]);
        let seen = w.monsters[0].unwrap().brain.seen;
        if seen != nowhere() {
            heard = Some(seen);
            break;
        }
    }
    let heard = heard.expect("a walk behind it was heard");
    assert!(
        sim::math::wide_flat_dist(heard, w.players[0].pos).raw() < Fx::from_int(2).raw(),
        "it sampled where the footfall was: {heard:?}, the fighter at {:?}",
        w.players[0].pos
    );
    assert!(noise::all(&w.lore).any(|n| n.kind == NoiseKind::Footfall && n.who == 0));
}

#[test]
fn a_crouch_walk_makes_no_noise_and_a_walk_does() {
    let mut w = watching(ArenaId::PROVING_GROUND, at(10, -10), at(-10, 10));
    for _ in 0..90 {
        w.advance([Input::aimed(Input::CROUCH | Input::W, 0), Input::default()]);
    }
    assert!(
        !noise::all(&w.lore).any(|n| n.who == 0 && n.kind == NoiseKind::Footfall),
        "crouched, nothing"
    );
    for _ in 0..30 {
        w.advance([Input::aimed(Input::W, 0), Input::default()]);
    }
    assert!(noise::all(&w.lore).any(|n| n.who == 0 && n.kind == NoiseKind::Footfall));
}

#[test]
fn a_landing_and_a_dodge_are_heard_and_the_ring_is_bounded() {
    let mut w = watching(ArenaId::PROVING_GROUND, at(10, -10), at(-10, 10));
    w.advance([Input::aimed(Input::SPACE, 0), Input::default()]);
    for _ in 0..60 {
        w.advance([Input::default(); MAX_PLAYERS]);
    }
    assert!(noise::all(&w.lore).any(|n| n.kind == NoiseKind::Landing));
    w.advance([Input::aimed(Input::SHIFT | Input::W, 0), Input::default()]);
    assert!(noise::all(&w.lore).any(|n| n.kind == NoiseKind::Dodge));
    for _ in 0..600 {
        w.advance([
            Input::aimed(Input::W, (w.frame * 300) as u16),
            Input::default(),
        ]);
    }
    assert!(noise::all(&w.lore).count() <= sentinel::FIGHT.layout.noises as usize);
}

#[test]
fn a_fight_that_does_not_hear_keeps_no_ring() {
    let mut w = World::hunt([Class::Champion; MAX_PLAYERS]);
    for _ in 0..120 {
        w.advance([Input::aimed(Input::W, 0); MAX_PLAYERS]);
    }
    assert_eq!(noise::all(&w.lore).count(), 0);
    assert!(w.lore.is_blank());
}

#[test]
fn the_glance_samples_nobody_it_is_told_it_cannot_perceive() {
    let mut m = Monster::new(SpeciesId::RIDGEBACK);
    m.brain.seen = nowhere();
    let q = [Quarry {
        pos: at(6, 0),
        vel: V3::ZERO,
        alive: true,
        aboard: false,
        stunned: false,
    }; MAX_PLAYERS];
    let blind = Senses {
        perceived: [false; MAX_PLAYERS],
        heard: None,
    };
    let ground = Terrain::bare(ArenaId::PROVING_GROUND.get());
    m.step_in(&q, &blind, &ground);
    assert_eq!(
        m.brain.seen,
        nowhere(),
        "what it cannot perceive it does not sample"
    );
    m.brain.glance_left = 0;
    m.step_in(&q, &Senses::ALL, &ground);
    assert_eq!(m.brain.seen, at(6, 0));
}

// ---------------------------------------------------------------------------
// A5: in view
// ---------------------------------------------------------------------------

fn scene_of(w: &World, ground: &Terrain, f: impl FnOnce(&Scene)) {
    let fighters = w.players;
    let effects = w.effects;
    let field = sim::stones::gather(&fighters);
    let scene = Scene {
        stones: &field,
        players: &fighters,
        effects: &effects,
        quarry: &w.monsters,
        critters: &w.critters,
        arena: ground,
    };
    f(&scene);
}

#[test]
fn in_view_is_on_the_screen_and_not_behind_anything() {
    let mut w = World::with_classes([Class::Champion; MAX_PLAYERS]);
    w.players[0].pos = at(-12, 10);
    let look = Input::aimed(0, 0);
    let cone = Fx::ratio(1, 9);
    let ground = w.terrain();
    scene_of(&w, &ground, |scene| {
        let ahead = V3::new(Fx::from_int(0), Fx::from_int(1), Fx::from_int(10));
        assert!(aim::in_view(0, look, ahead, cone, scene));
        let behind = V3::new(Fx::from_int(-25), Fx::from_int(1), Fx::from_int(10));
        assert!(!aim::in_view(0, look, behind, cone, scene));
        let aside = V3::new(Fx::from_int(-11), Fx::from_int(1), Fx::from_int(0));
        assert!(
            !aim::in_view(0, look, aside, cone, scene),
            "far off to one side"
        );
        // Low behind a platform (x from 5 to 9, z within 4) is hidden; above
        // it is seen.
        let mut w2 = w.clone();
        w2.players[0].pos = at(-4, 0);
        let fighters = w2.players;
        let field = sim::stones::gather(&fighters);
        let s2 = Scene {
            players: &fighters,
            stones: &field,
            ..*scene
        };
        let low = V3::new(Fx::from_int(11), Fx::ratio(1, 4), Fx::ZERO);
        assert!(!aim::in_view(0, look, low, cone, &s2));
        let high = V3::new(Fx::from_int(11), Fx::from_int(4), Fx::ZERO);
        assert!(aim::in_view(0, look, high, cone, &s2));
    });
    // A cloud between hides it as well.
    let mut w = World::hunt_of([Class::Champion; MAX_PLAYERS], SpeciesId::SENTINEL);
    w.lore.set_word(sentinel::word::LAID, 1);
    w.players[0].pos = at(-12, 10);
    hazard::place(
        &mut w.lore,
        Hazard::disc(sentinel::SMOKE, at(-4, 10), Fx::from_int(3)),
    );
    let ground = w.terrain();
    scene_of(&w, &ground, |scene| {
        let ahead = V3::new(Fx::from_int(4), Fx::from_int(1), Fx::from_int(10));
        assert!(!aim::in_view(0, look, ahead, cone, scene));
    });
}

// ---------------------------------------------------------------------------
// A creature against the arena's solids
// ---------------------------------------------------------------------------

#[test]
fn a_creature_that_collides_walks_into_solids_and_one_that_does_not_walks_through() {
    // The proving ground's platform runs from x 5 to 9, z within 4, and is
    // taller than the sentinel steps over.
    let ground = Terrain::bare(ArenaId::PROVING_GROUND.get());
    let walk = |species: SpeciesId| {
        let mut m = Monster::new(species);
        m.pos = at(-2, 0);
        m.yaw = Fx::ZERO;
        m.brain.grace = 0;
        let q = [Quarry {
            pos: at(12, 0),
            vel: V3::ZERO,
            alive: true,
            aboard: false,
            stunned: false,
        }; MAX_PLAYERS];
        let mut furthest = m.pos.x;
        for _ in 0..240 {
            m.brain.think_left = 1;
            m.step_in(&q, &Senses::ALL, &ground);
            furthest = furthest.max(m.pos.x);
        }
        furthest
    };
    let radius = sentinel::SPECIES.fight_fx(sim::species::FightField::BodyRadius);
    let sentinel_got = walk(SpeciesId::SENTINEL);
    assert!(
        sentinel_got.raw() <= Fx::from_int(5).sub(radius).raw(),
        "the sentinel reached {sentinel_got:?}, through the platform"
    );
    let ridgeback_got = walk(SpeciesId::RIDGEBACK);
    assert!(
        ridgeback_got.raw() > Fx::from_int(5).sub(radius).add(Fx::ONE).raw(),
        "the Ridgeback got to {ridgeback_got:?}: it walks through solids, as it always has"
    );
}

#[test]
fn a_charge_into_a_solid_calls_the_species_hook() {
    let ground = Terrain::bare(ArenaId::PROVING_GROUND.get());
    let mut m = Monster::new(SpeciesId::SENTINEL);
    let radius = sentinel::SPECIES.fight_fx(sim::species::FightField::BodyRadius);
    m.pos = V3::new(
        Fx::from_int(5).sub(radius).sub(Fx::ratio(1, 10)),
        Fx::ZERO,
        Fx::ZERO,
    );
    m.yaw = Fx::ZERO;
    m.speed = Fx::from_int(20);
    m.doing = Doing::Active {
        kind: ridgeback::CHARGE,
        left: 20,
    };
    m.brain.grace = 0;
    let q = [Quarry::default(); MAX_PLAYERS];
    m.step_in(&q, &Senses::ALL, &ground);
    assert!(matches!(m.doing, Doing::Flinch { .. }), "{:?}", m.doing);
}
