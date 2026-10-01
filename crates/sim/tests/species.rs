//! Species as data, and a world with more than one creature in it.
//!
//! The creature machinery reads everything about an animal from its species'
//! table (`sim::species`, `docs/design/species.md`). These check the two
//! promises that rest on: **every registered table is well formed**, so a new
//! species that gets something wrong is told so here rather than by a panic
//! in the middle of a rollback; and **the world's second creature slot is a
//! real one** -- the Pair need it -- with its own rider, its own hits, and a
//! hunt that is not over until both are down.

use sim::beast::{MAX_BONES, MAX_BREAKABLE, MAX_PARTS, NO_PARENT, channels};
use sim::fixed::Fx;
use sim::monster::{Doing, MAX_MONSTERS, Monster, mount_part, mount_slot};
use sim::species::{self, MAX_MOVES, SpeciesId, ridgeback};
use sim::state::{MAX_PLAYERS, Phase, QUARRY};
use sim::{Class, Input, V3, World};

// ---------------------------------------------------------------------------
// The tables
// ---------------------------------------------------------------------------

#[test]
fn every_registered_species_is_a_well_formed_table() {
    let mut seen = 0;
    for sp in species::all() {
        seen += 1;
        let name = sp.name;
        assert_eq!(
            sp.id.get().name,
            name,
            "{name} is registered under another id"
        );

        // The skeleton: parents before children, one root, and it is first.
        assert!(
            !sp.bones.is_empty() && sp.bones.len() <= MAX_BONES,
            "{name}: bones"
        );
        assert_eq!(
            sp.bones[0].parent, NO_PARENT,
            "{name}: bone zero is not the root"
        );
        for (b, bone) in sp.bones.iter().enumerate().skip(1) {
            assert!(
                bone.parent < b,
                "{name}: bone {} hangs off one that comes after it",
                bone.name
            );
        }
        for (l, r) in sp.mirror {
            assert!(*l < sp.bones.len() && *r < sp.bones.len(), "{name}: mirror");
            assert_eq!(
                sp.bones[*l].side, -sp.bones[*r].side,
                "{name}: a mirrored pair on one side"
            );
        }
        assert!(sp.neck.iter().all(|b| *b < sp.bones.len()), "{name}: neck");
        assert!(
            !sp.follows.is_empty(),
            "{name}: no bone for the body's hits to ride"
        );
        assert!(
            sp.follows.iter().all(|b| *b < sp.bones.len()),
            "{name}: follows"
        );

        // The body.
        assert!(
            !sp.parts.is_empty() && sp.parts.len() <= MAX_PARTS,
            "{name}: parts"
        );
        for p in sp.parts {
            assert!(
                p.shape.bone < sp.bones.len(),
                "{name}: {} hangs off no bone",
                p.name
            );
            assert!(
                (p.vuln as usize) < sp.own.len(),
                "{name}: {} has no hide knob",
                p.name
            );
            let (lo, hi) = (p.shape.min, p.shape.max);
            assert!(
                lo.x.raw() < hi.x.raw() && lo.y.raw() < hi.y.raw() && lo.z.raw() < hi.z.raw(),
                "{name}: {} is inside out",
                p.name
            );
        }
        let breakable = sp.parts.iter().filter(|p| p.shape.breakable).count();
        assert!(
            breakable <= MAX_BREAKABLE,
            "{name}: too many breakable parts"
        );
        assert_eq!(sp.breakables().count(), breakable, "{name}: breakables");
        for leg in sp.legs {
            assert!(
                leg.upper < sp.parts.len() && leg.foot < sp.parts.len(),
                "{name}: leg"
            );
            assert!(
                leg.hip < sp.bones.len() && leg.knee < sp.bones.len(),
                "{name}: leg"
            );
            assert!(
                leg.side == -1 || leg.side == 1,
                "{name}: a leg on neither side"
            );
        }

        // What it does.
        assert!(
            !sp.moves.is_empty() && sp.moves.len() <= MAX_MOVES,
            "{name}: moves"
        );
        for m in sp.moves {
            assert!(m.clip < sp.clips.len(), "{name}: {} plays no clip", m.name);
            assert!(
                sp.clips[m.clip].phased,
                "{name}: {}'s clip is not phased",
                m.name
            );
            if let Some(k) = m.scaled_by {
                assert!(
                    (k as usize) < sp.own.len(),
                    "{name}: {} scaled by no knob",
                    m.name
                );
            }
        }
        let stock = sp.stock;
        for clip in [
            stock.idle,
            stock.walk,
            stock.gallop,
            stock.flinch,
            stock.stumble,
            stock.topple,
            stock.dead,
        ] {
            assert!(
                clip < sp.clips.len(),
                "{name}: a stock clip it does not have"
            );
        }

        // Its baked table: one span per clip, every span inside the table,
        // every row as wide as the skeleton.
        assert_eq!(sp.span.len(), sp.clips.len(), "{name}: re-bake its clips");
        for (start, count) in sp.span {
            assert!(
                (*start as usize + *count as usize) <= sp.rows,
                "{name}: a span past the table"
            );
        }
        for r in 0..sp.rows {
            assert_eq!(
                (sp.row)(r).len(),
                channels(sp.bones.len()),
                "{name}: row {r}"
            );
        }

        // Its knobs: what `tests/oven.rs` checks against the bake, checked
        // here against the table.
        assert_eq!(
            sp.tuned.len(),
            sp.knob_count(),
            "{name}: re-bake its tuning"
        );
    }
    assert!(seen >= 1, "no species is registered");
}

#[test]
fn every_registered_species_stands_and_moves() {
    // The smallest thing a new species has to do: be built, stand up on its
    // own skeleton, take a step and a hit, without a panic.
    for sp in species::all() {
        let mut w = World::hunt_of([Class::Champion; MAX_PLAYERS], sp.id);
        for _ in 0..240 {
            w.advance([Input::default(); MAX_PLAYERS]);
        }
        let beast = w.monster().copied().expect("a hunt has a creature");
        assert_eq!(beast.species, sp.id);
        let rig = beast.rig();
        assert!(rig.origin.y.raw() == 0, "{} is off the floor", sp.name);
        let mut hurt = beast;
        hurt.take_hit(0, 100);
        assert!(hurt.health < beast.health, "{} cannot be hurt", sp.name);
    }
}

// ---------------------------------------------------------------------------
// Two creatures
// ---------------------------------------------------------------------------

fn pair() -> World {
    World::hunt_with(
        [Class::Champion; MAX_PLAYERS],
        [Some(SpeciesId::RIDGEBACK); MAX_MONSTERS],
    )
}

/// Hold a creature still in its walking state, past the moment it takes to
/// notice anybody.
fn hold(w: &mut World, slot: usize) {
    let beast = w.monsters[slot].as_mut().expect("a creature in that slot");
    beast.doing = Doing::Prowl;
    beast.brain.think_left = u16::MAX;
    beast.brain.grace = 0;
}

#[test]
fn two_creatures_stand_apart_and_both_act() {
    let mut w = pair();
    let (a, b) = (w.monsters[0].unwrap(), w.monsters[1].unwrap());
    let apart = V3::new(a.pos.x.sub(b.pos.x), Fx::ZERO, a.pos.z.sub(b.pos.z)).flat_len();
    assert!(apart.raw() > 0, "two creatures spawned on one spot");
    let mut started = [false; MAX_MONSTERS];
    for _ in 0..1200 {
        w.advance([Input::default(); MAX_PLAYERS]);
        for (slot, beast) in w.monsters.iter().enumerate() {
            started[slot] |= beast.is_some_and(|b| matches!(b.doing, Doing::Startup { .. }));
        }
    }
    assert_eq!(
        started, [true; MAX_MONSTERS],
        "a creature never moved: {started:?}"
    );
}

#[test]
fn a_world_with_two_creatures_replays_exactly() {
    let script: Vec<[Input; MAX_PLAYERS]> = (0..600u32)
        .map(|i| {
            let r = i.wrapping_mul(0x9E37_79B9);
            [
                Input::aimed((r & 0x1ff) as u16, (r >> 16) as u16),
                Input::default(),
            ]
        })
        .collect();
    let run = |from: World, inputs: &[[Input; MAX_PLAYERS]]| {
        let mut w = from;
        for i in inputs {
            w.advance(*i);
        }
        w
    };
    let half = run(pair(), &script[..300]);
    let whole = run(half.clone(), &script[300..]);
    assert_eq!(run(half, &script[300..]).checksum(), whole.checksum());
    assert_eq!(run(pair(), &script).checksum(), whole.checksum());
}

#[test]
fn a_rider_on_the_second_creature_rides_the_second_creature() {
    let mut w = pair();
    hold(&mut w, 0);
    hold(&mut w, 1);
    // Onto the second one's barrel, the way landing on it would.
    let beast = w.monsters[1].unwrap();
    let shape = beast.sp().shape(ridgeback::BARREL);
    let mid = shape.min.add(shape.max).scale(Fx::ratio(1, 2));
    let spot = V3::new(mid.x, shape.max.y.add(Fx::ratio(1, 8)), mid.z);
    w.players[0].pos = beast.world_of(ridgeback::BARREL, spot);
    w.players[0].vel = V3::new(Fx::ZERO, Fx::ratio(-1, 1), Fx::ZERO);
    w.players[0].grounded = false;
    w.advance([Input::default(); MAX_PLAYERS]);
    assert!(w.players[0].aboard(), "did not land on it");
    assert_eq!(
        mount_slot(w.players[0].mount),
        1,
        "landed on the wrong creature"
    );
    assert_eq!(mount_part(w.players[0].mount), ridgeback::BARREL);

    // Walk the second one forward. The rider goes with it, and the first one
    // standing still has nothing to say about where they are.
    for _ in 0..90 {
        hold(&mut w, 0);
        hold(&mut w, 1);
        let b = w.monsters[1].as_mut().unwrap();
        b.speed = b.sp().walk();
        w.advance([Input::default(); MAX_PLAYERS]);
    }
    let beast = w.monsters[1].unwrap();
    let under = beast.world_of(ridgeback::BARREL, w.players[0].local);
    assert!(w.players[0].aboard(), "fell off a walking creature");
    assert_eq!(
        w.players[0].pos, under,
        "the rider is not where the second creature carried them"
    );
}

#[test]
fn a_hit_on_one_creature_is_a_hit_on_that_one() {
    let mut a = pair();
    let mut b = pair();
    a.monsters[0]
        .as_mut()
        .unwrap()
        .take_hit(ridgeback::RIDGE, 200);
    b.monsters[1]
        .as_mut()
        .unwrap()
        .take_hit(ridgeback::RIDGE, 200);
    let full = Monster::new(SpeciesId::RIDGEBACK).health;
    assert!(a.monsters[0].unwrap().health < full && a.monsters[1].unwrap().health == full);
    assert!(b.monsters[1].unwrap().health < full && b.monsters[0].unwrap().health == full);
}

#[test]
fn the_hunt_is_not_over_until_every_creature_is_down() {
    let mut w = pair();
    w.monsters[0].as_mut().unwrap().health = 0;
    for _ in 0..30 {
        w.advance([Input::default(); MAX_PLAYERS]);
    }
    assert!(
        matches!(w.phase, Phase::Fighting),
        "the hunt ended with one creature still standing"
    );
    w.monsters[1].as_mut().unwrap().health = 0;
    w.advance([Input::default(); MAX_PLAYERS]);
    match w.phase {
        Phase::RoundOver { winner, .. } => assert_ne!(winner, QUARRY),
        Phase::Fighting => panic!("both creatures down and the hunt went on"),
    }
}
