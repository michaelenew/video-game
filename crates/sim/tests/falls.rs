//! Falling (bestiary P6): one rule for the whole game, keyed to the height of
//! the last thing stood on rather than to the landing speed.
//!
//! What is pinned: the rule's arithmetic; that a fighter's own jump, however
//! high, is not a fall; that a push up on the way down starts the fall again;
//! that a slow landing is halved; that a carried fighter's fall starts where
//! they were let go; a real fall off the range's tower; and the reason the
//! pinned hunts did not move -- nothing in the proving ground stands higher
//! than a fall is free, the Ridgeback's back at the top of its rear included.

use sim::arena::{ArenaId, proving_ground};
use sim::beast::{Rig, sample};
use sim::fixed::Fx;
use sim::species::ridgeback;
use sim::state::{MAX_PLAYERS, Player, fall_rule};
use sim::tuning as t;
use sim::{Class, Input, V3, World};

fn high(y: i32) -> V3 {
    V3::new(Fx::ZERO, Fx::from_int(y), Fx::ZERO)
}

/// The damage a fall of `metres` past the free height is worth.
fn worth(metres: Fx) -> i32 {
    metres.mul(Fx::from_int(t::fall_per_metre())).to_int()
}

#[test]
fn nothing_in_the_proving_ground_stands_higher_than_a_fall_is_free() {
    // The highest top face of any part a rider can stand on, in any frame of
    // any clip the Ridgeback has: its shoulders at the top of the rear.
    let sp = &ridgeback::SPECIES;
    let mut highest = Fx::ZERO;
    for clip in 0..sp.clips.len() {
        for i in 0..=64 {
            let pose = sample(sp, clip, Fx::ratio(i, 64));
            let rig = Rig::build(sp, V3::ZERO, Fx::ZERO, &pose);
            for (part, p) in sp.parts.iter().enumerate() {
                if p.shape.mountable {
                    for corner in rig.top_face(part) {
                        highest = highest.max(corner.y);
                    }
                }
            }
        }
    }
    // A stone stood on a platform is the highest thing anybody builds.
    let platforms = proving_ground::wall_height();
    let stones = platforms.add(t::structure_height());
    let free = t::fall_free();
    assert!(
        highest.raw() < free.raw() && stones.raw() < free.raw(),
        "the Ridgeback lifts a rider to {highest:?} and a stone stands at {stones:?}, \
         and a fall is free up to {free:?}: the pinned hunts would take fall damage"
    );
    assert!(
        highest.raw() > Fx::from_int(8).raw(),
        "the rear is the highest thing there is"
    );
}

#[test]
fn a_fighter_s_own_jump_is_not_a_fall_however_high() {
    let mut was = Player::new(Class::Champion);
    was.grounded = true;
    let mut p = was;
    fall_rule(&was, &mut p);
    // Launched to twenty metres from the floor and back.
    for y in [5, 12, 20] {
        let before = p;
        p.grounded = false;
        p.pos = high(y);
        p.vel = V3::new(Fx::ZERO, Fx::from_int(10), Fx::ZERO);
        assert_eq!(fall_rule(&before, &mut p), 0);
    }
    let before = p;
    p.pos = V3::ZERO;
    p.vel = V3::new(Fx::ZERO, Fx::from_int(-18), Fx::ZERO);
    p.grounded = true;
    assert_eq!(
        fall_rule(&before, &mut p),
        0,
        "it left the floor, it lands on the floor"
    );
    assert_eq!(p.fall_over, Fx::ZERO);
}

#[test]
fn the_rule_pays_for_what_is_past_the_free_height_and_below_the_landing() {
    let free = t::fall_free();
    let stood = free.add(Fx::from_int(6));
    let mut p = Player::new(Class::Champion);
    p.grounded = true;
    p.pos = V3::new(Fx::ZERO, stood, Fx::ZERO);
    let was = p;
    fall_rule(&was, &mut p);
    assert_eq!(p.fall_over, Fx::from_int(6));
    // Off the edge...
    let was = p;
    p.grounded = false;
    p.pos.y = stood.sub(Fx::ONE);
    p.vel.y = Fx::from_int(-5);
    assert_eq!(fall_rule(&was, &mut p), 0);
    // ...onto a ledge two metres up: six past the free height, less the two.
    let was = p;
    p.vel.y = Fx::from_int(-18);
    let was = Player { vel: p.vel, ..was };
    p.grounded = true;
    p.pos.y = Fx::from_int(2);
    assert_eq!(fall_rule(&was, &mut p), worth(Fx::from_int(4)));
}

#[test]
fn a_push_up_on_the_way_down_starts_the_fall_again() {
    let mut p = Player::new(Class::DualMage);
    p.falling_from(t::fall_free().add(Fx::from_int(11)));
    p.grounded = false;
    // A wing beat at three metres past the free height.
    let was = p;
    p.pos.y = t::fall_free().add(Fx::from_int(3));
    p.vel.y = Fx::from_int(8);
    fall_rule(&was, &mut p);
    assert_eq!(p.fall_over, Fx::from_int(3));
    // It never raises it: a beat higher up does not add to the fall.
    let was = p;
    p.pos.y = t::fall_free().add(Fx::from_int(7));
    fall_rule(&was, &mut p);
    assert_eq!(p.fall_over, Fx::from_int(3));
    // Landing hard on the floor pays for the three.
    let was = Player {
        vel: V3::new(Fx::ZERO, Fx::from_int(-18), Fx::ZERO),
        ..p
    };
    p.pos.y = Fx::ZERO;
    p.grounded = true;
    assert_eq!(fall_rule(&was, &mut p), worth(Fx::from_int(3)));
}

#[test]
fn a_slow_landing_is_halved() {
    let mut p = Player::new(Class::DualMage);
    p.falling_from(t::fall_free().add(Fx::from_int(4)));
    p.grounded = false;
    let slow = t::fall_soft().sub(Fx::ONE).neg();
    let was = Player {
        vel: V3::new(Fx::ZERO, slow, Fx::ZERO),
        ..p
    };
    p.grounded = true;
    p.pos = V3::ZERO;
    assert_eq!(fall_rule(&was, &mut p), worth(Fx::from_int(4)) / 2);
}

#[test]
fn walking_off_the_range_tower_costs_what_the_rule_says() {
    let mut w = World::versus_in([Class::Champion; MAX_PLAYERS], ArenaId::RANGE);
    // On the tower's top (12 m, x 17 to 23, z within 3), at the corner whose
    // two neighbours are open floor, walking out over it.
    w.players[0].pos = V3::new(Fx::ratio(228, 10), Fx::from_int(12), Fx::ratio(28, 10));
    w.players[0].grounded = true;
    w.players[1].pos = V3::new(Fx::from_int(-100), Fx::ZERO, Fx::ZERO);
    let full = w.players[0].health;
    let diagonal = 8192; // an eighth of a turn: out across the corner
    let mut landed = false;
    for _ in 0..240 {
        w.advance([Input::aimed(Input::W, diagonal), Input::default()]);
        if w.players[0].pos.y.raw() == 0 && w.players[0].grounded {
            landed = true;
            break;
        }
    }
    assert!(landed, "it never came down: {:?}", w.players[0].pos);
    let want = worth(Fx::from_int(12).sub(t::fall_free()));
    assert_eq!(full - w.players[0].health, want, "a twelve-metre fall");
}

#[test]
fn a_fall_in_the_proving_ground_hashes_as_it_always_did() {
    // The excess is zero for every fall that cannot hurt, and the hash writes
    // it only when it is not: a hunt with jumps in it hashes the same with or
    // without the field.
    let mut w = World::hunt([Class::DualMage; MAX_PLAYERS]);
    for f in 0..600u32 {
        let bits = if f % 40 < 3 { Input::SPACE } else { Input::W };
        w.advance([Input::aimed(bits, (f * 97) as u16); MAX_PLAYERS]);
        for p in &w.players {
            assert_eq!(p.fall_over, Fx::ZERO);
        }
    }
}
