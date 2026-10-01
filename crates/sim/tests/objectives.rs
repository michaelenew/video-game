//! Objectives (bestiary P7): something besides the hunters that can lose.
//! Stood up on the dev creature, the sentinel, in the range, whose two sites
//! hold its gate and its cart.
//!
//! What is pinned: they stand where the arena says, whole; the cart rolls only
//! while a hunter is near it and wins the hunt when it arrives; both are solids
//! bodies stop at; a creature's blow lands on one once a move, for what the
//! move does times what it takes; and a broken one that loses the hunt loses
//! it.

use sim::arena::{ArenaId, range};
use sim::fixed::Fx;
use sim::monster::Doing;
use sim::objective::{self, ObjectiveField};
use sim::species::{SpeciesId, ridgeback, sentinel};
use sim::state::{MAX_PLAYERS, Phase, QUARRY};
use sim::{Class, Input, V3, World};

fn in_the_range() -> World {
    let mut w = World::hunt_in(
        [Class::Champion; MAX_PLAYERS],
        [Some(SpeciesId::SENTINEL), None],
        ArenaId::RANGE,
    );
    w.lore.set_word(sentinel::word::LAID, 1);
    park(&mut w);
    w
}

/// The creature far off and taking no notice.
fn park(w: &mut World) {
    if let Some(m) = w.monsters[0].as_mut() {
        m.pos = V3::new(Fx::from_int(-60), Fx::ZERO, Fx::ZERO);
        m.brain.grace = u16::MAX;
    }
}

fn step(w: &mut World, frames: u32) {
    for _ in 0..frames {
        park(w);
        w.advance([Input::default(); MAX_PLAYERS]);
        for p in w.players.iter_mut() {
            p.health = p.full_health();
        }
    }
}

fn cart(w: &World) -> objective::Standing {
    objective::standing(&w.lore, w.arena())
        .find(|o| o.index == sentinel::CART)
        .expect("the cart stands in the range")
}

#[test]
fn they_stand_where_the_arena_says_and_whole() {
    let w = in_the_range();
    let standing: Vec<_> = objective::standing(&w.lore, w.arena()).collect();
    assert_eq!(standing.len(), 2);
    for o in &standing {
        assert_eq!(o.state.health, o.full);
        assert_eq!(
            o.full,
            objective::stat(&sentinel::SPECIES, o.index, ObjectiveField::Health)
        );
        let (x, z) = o.site.route[0];
        assert_eq!(
            o.at,
            V3::new(Fx::ratio(x, 100), Fx::ZERO, Fx::ratio(z, 100))
        );
    }
    // Where the arena has no site for them, there are none.
    let home = World::hunt_of([Class::Champion; MAX_PLAYERS], SpeciesId::SENTINEL);
    assert_eq!(objective::standing(&home.lore, home.arena()).count(), 0);
}

#[test]
fn the_cart_rolls_only_while_a_hunter_is_near_it() {
    let mut w = in_the_range();
    w.players[0].pos = V3::new(Fx::from_int(-100), Fx::ZERO, Fx::ZERO);
    w.players[1].pos = V3::new(Fx::from_int(-100), Fx::ZERO, Fx::from_int(3));
    step(&mut w, 60);
    assert_eq!(cart(&w).state.along, 0, "nobody near it, it stays put");

    let start = cart(&w).at;
    w.players[0].pos = start.add(V3::new(Fx::ZERO, Fx::ZERO, Fx::from_int(3)));
    step(&mut w, 60);
    let speed = objective::stat_fx(&sentinel::SPECIES, sentinel::CART, ObjectiveField::Speed);
    let rolled = cart(&w).state.along();
    assert!(
        rolled.sub(speed).abs().raw() < Fx::ratio(1, 10).raw(),
        "a second escorted at {speed:?} m/s rolled {rolled:?}"
    );
}

#[test]
fn the_cart_arriving_wins_the_hunt() {
    let mut w = in_the_range();
    let site = &range::ARENA.sites[1];
    let mut o = objective::get(&w.lore, sentinel::CART);
    let end = site.length();
    o.along = end.sub(Fx::ratio(1, 5)).raw() as u32;
    objective::set(&mut w.lore, sentinel::CART, o);
    w.players[0].pos = cart(&w)
        .at
        .add(V3::new(Fx::ZERO, Fx::ZERO, Fx::from_int(2)));
    step(&mut w, 30);
    assert!(objective::won(&w.lore));
    assert!(
        matches!(w.phase, Phase::RoundOver { winner: 0, .. }),
        "{:?}",
        w.phase
    );
}

#[test]
fn a_broken_gate_loses_the_hunt() {
    let mut w = in_the_range();
    let full = objective::get(&w.lore, sentinel::GATE).health;
    let dealt = objective::strike(&mut w.lore, sentinel::GATE, full + 10);
    assert_eq!(dealt, full);
    assert!(objective::lost(&w.lore));
    step(&mut w, 1);
    assert!(matches!(w.phase, Phase::RoundOver { winner, .. } if winner == QUARRY));
}

#[test]
fn they_are_solids_bodies_stop_at() {
    let w = in_the_range();
    let ground = w.terrain();
    assert_eq!(ground.raised().len(), 2);
    let gate = objective::standing(&w.lore, w.arena())
        .find(|o| o.index == sentinel::GATE)
        .unwrap();
    let face = gate.at.x.sub(gate.site.extent().x);
    // Walking into its face from the west.
    let r = ground.resolve(
        V3::new(face, Fx::ZERO, gate.at.z),
        V3::new(Fx::from_int(5), Fx::ZERO, Fx::ZERO),
        true,
    );
    assert!(r.wall);
    assert!(r.pos.x.raw() < face.raw());
}

#[test]
fn a_creature_s_blow_lands_on_it_once_a_move() {
    let mut w = in_the_range();
    let gate = objective::standing(&w.lore, w.arena())
        .find(|o| o.index == sentinel::GATE)
        .unwrap();
    let full = gate.state.health;
    // Find where, facing the gate, the bite reaches it -- with the move out,
    // on the frame it will be on when the world asks.
    let bite = sentinel::SPECIES.attack(ridgeback::BITE);
    let out = bite.active.max(3);
    // No nearer than a metre off the gate's face and the creature's body: the
    // gate is a solid, the sentinel walks into solids rather than through
    // them, and a move that runs it into one is knocked out of its stride
    // (its `bumped`) -- a bite that lunged into the gate would never land.
    let body = sentinel::SPECIES.fight_fx(sim::species::FightField::BodyRadius);
    let nearest = gate.site.extent().x.add(body).add(Fx::ONE);
    let mut found = None;
    for back in 0..40 {
        let mut m = w.monsters[0].unwrap();
        let back = nearest.add(Fx::ratio(back, 4));
        m.pos = gate.at.sub(V3::new(back, Fx::ZERO, Fx::ZERO));
        m.yaw = Fx::ZERO;
        m.doing = Doing::Active {
            kind: ridgeback::BITE,
            left: out - 1,
        };
        if m.reaches(gate.at, gate.site.extent().y, gate.radius()) {
            found = Some(m.pos);
            break;
        }
    }
    let at = found.expect("somewhere the bite reaches the gate");
    for _ in 0..3 {
        let m = w.monsters[0].as_mut().unwrap();
        m.pos = at;
        m.yaw = Fx::ZERO;
        m.brain.grace = u16::MAX;
        m.doing = Doing::Active {
            kind: ridgeback::BITE,
            left: out,
        };
        w.advance([Input::default(); MAX_PLAYERS]);
    }
    let after = objective::get(&w.lore, sentinel::GATE);
    assert_eq!(
        full - after.health,
        bite.damage,
        "one bite, three frames of it, one blow"
    );
    assert_eq!(after.taken, bite.damage);
}
