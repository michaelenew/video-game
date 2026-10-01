//! The Pair: the rules `docs/design/creatures/the-pair.md` pins, as sentences.
//!
//! Most of these set up one cat (the other slot emptied, so the pair brain
//! plays it alone) in front of one fighter, start a move, and play the answer
//! the design gives -- and the wrong one -- with the fighter's buttons.

use sim::fixed::Fx;
use sim::monster::{Doing, Monster};
use sim::species::SpeciesId;
use sim::species::pair::{self, Knob, fight};
use sim::state::{MAX_PLAYERS, Player};
use sim::{Class, Input, V3, World};

/// A hunt against the Pair, the second fighter out of it.
fn hunt() -> World {
    let mut w = World::hunt_of([Class::Champion; MAX_PLAYERS], SpeciesId::PAIR);
    w.players[1].health = 0;
    w
}

/// Where the duels are fought: a clear strip of the Den, south of the
/// platforms and between the stones.
fn base() -> V3 {
    V3::new(Fx::from_int(-4), Fx::ZERO, Fx::from_int(-9))
}

/// A point `x` metres east and `z` south of [`base`].
fn at(x: i32, z: i32) -> V3 {
    base().add(V3::new(Fx::from_int(x), Fx::ZERO, Fx::from_int(z)))
}

/// One cat at [`base`] facing +x, a fighter `at` metres down +x facing it,
/// and the other cat gone. Both awake.
fn duel(dist: i32) -> World {
    let mut w = hunt();
    w.monsters[1] = None;
    let m = w.monsters[0].as_mut().unwrap();
    m.pos = base();
    m.yaw = Fx::ZERO;
    m.brain.grace = 0;
    m.brain.think_left = 600;
    w.players[0].pos = at(dist, 0);
    w.players[0].facing = V3::new(Fx::ONE.neg(), Fx::ZERO, Fx::ZERO);
    m.brain.seen = w.players[0].pos;
    // Settle: a few frames of nothing so the pair brain has written its
    // first lore and the cat has glanced.
    for _ in 0..10 {
        w.advance([idle(); MAX_PLAYERS]);
        hold_still(&mut w);
    }
    w
}

/// Keep the cat where the test put it, thinking about nothing.
fn hold_still(w: &mut World) {
    if let Some(m) = w.monsters[0].as_mut() {
        m.brain.think_left = m.brain.think_left.max(60);
        if m.doing.free() {
            m.pos = base();
            m.yaw = Fx::ZERO;
            m.yaw_rate = Fx::ZERO;
            m.speed = Fx::ZERO;
        }
    }
}

/// Looking at the cat (down -x), no keys.
fn idle() -> Input {
    Input::aimed(0, 0x8000)
}

fn cat(w: &World) -> Monster {
    w.monsters[0].unwrap()
}

fn me(w: &World) -> Player {
    w.players[0]
}

/// Start a move on the cat now, as its brain would.
fn throw(w: &mut World, kind: u8) {
    let m = w.monsters[0].as_mut().unwrap();
    m.doing = Doing::Startup {
        kind,
        left: pair::SPECIES.attack(kind).startup,
    };
    m.hit_used = false;
    m.brain.think_left = 600;
    if pair::MOVES[kind as usize].lobbed {
        m.lob(kind);
    }
}

/// Play the move out: `press` from frame `at` of it, for `hold` frames.
/// Returns the fighter's health lost.
fn play(w: &mut World, kind: u8, at: u32, press: Input, hold: u32) -> i32 {
    let before = me(w).health;
    let total = pair::SPECIES.attack(kind).total() as u32;
    for f in 0..total + 4 {
        let input = if f >= at && f < at + hold { press } else { idle() };
        w.advance([input, Input::default()]);
        if let Some(m) = w.monsters[0].as_mut() {
            m.brain.think_left = m.brain.think_left.max(60);
        }
    }
    before - me(w).health
}

#[test]
fn two_cats_are_hunted_together() {
    let w = World::hunt_of([Class::Champion; MAX_PLAYERS], SpeciesId::PAIR);
    assert!(w.monsters[0].is_some_and(|m| m.species == SpeciesId::PAIR));
    assert!(w.monsters[1].is_some_and(|m| m.species == SpeciesId::PAIR));
}

#[test]
fn the_pounce_is_dodged_toward_not_away() {
    // The fighter sees the flick on frame thirteen, fifteen frames late: it
    // dodges on frame twenty-eight, after the leap has left the ground.
    let at = 13 + 15;
    let toward = Input::aimed(Input::SHIFT | Input::W, 0x8000);
    let away = Input::aimed(Input::SHIFT | Input::S, 0x8000);

    let mut w = duel(7);
    throw(&mut w, pair::POUNCE);
    let lost = play(&mut w, pair::POUNCE, at, toward, 2);
    assert_eq!(lost, 0, "a dodge toward it, under the arc, is the answer");

    let mut w = duel(7);
    throw(&mut w, pair::POUNCE);
    let lost = play(&mut w, pair::POUNCE, at, away, 2);
    assert!(lost > 0, "a dodge away is caught by the skid");

    let mut w = duel(7);
    throw(&mut w, pair::POUNCE);
    let lost = play(&mut w, pair::POUNCE, at, idle(), 2);
    assert!(lost > 0, "standing still is the pounce landing");
}

#[test]
fn the_ambush_is_stepped_out_of_sideways() {
    // The fighter seven metres on with their back to it, walking sideways
    // from the moment a person could have seen the lane.
    let at = 15;
    let mut w = duel(7);
    throw(&mut w, pair::AMBUSH);
    let lost = play(&mut w, pair::AMBUSH, at, Input::aimed(Input::D, 0), 60);
    assert_eq!(lost, 0, "walking at right angles clears the lane");

    let mut w = duel(7);
    throw(&mut w, pair::AMBUSH);
    let lost = play(&mut w, pair::AMBUSH, at, Input::aimed(Input::W, 0), 60);
    assert!(lost > 0, "running along it does not");
}

#[test]
fn the_tail_trip_is_jumped() {
    let beside = |w: &mut World| {
        w.players[0].pos = at(-2, 2);
        let m = w.monsters[0].as_mut().unwrap();
        m.brain.seen = w.players[0].pos;
    };
    let mut w = duel(3);
    beside(&mut w);
    throw(&mut w, pair::TRIP);
    let at = pair::SPECIES.attack(pair::TRIP).startup as u32 - 6;
    let lost = play(&mut w, pair::TRIP, at, Input::aimed(Input::SPACE, 0), 30);
    assert_eq!(lost, 0, "a jump clears the low sweep");

    let mut w = duel(3);
    beside(&mut w);
    throw(&mut w, pair::TRIP);
    let lost = play(&mut w, pair::TRIP, 0, idle(), 0);
    assert!(lost > 0, "standing beside its hips is tripped");
}

#[test]
fn the_feint_and_the_pounce_differ_by_frame_ten() {
    let pose_at = |kind: u8, frame: u16| {
        let mut m = Monster::new(SpeciesId::PAIR);
        let a = pair::SPECIES.attack(kind);
        m.doing = Doing::Startup {
            kind,
            left: a.startup - frame,
        };
        m.pose()
    };
    // The tail's lift, all four bones of it.
    let tail = |p: &sim::beast::Pose| {
        [
            pair::bones::TAIL1,
            pair::bones::TAIL2,
            pair::bones::TAIL3,
            pair::bones::TAIL4,
        ]
        .iter()
        .fold(Fx::ZERO, |sum, b| sum.add(p.bone[*b].x))
    };
    for f in 0..=10 {
        let (p, q) = (pose_at(pair::POUNCE, f), pose_at(pair::FEINT, f));
        let d = tail(&p).sub(tail(&q)).abs();
        assert!(
            d.raw() < Fx::ratio(1, 100).raw(),
            "frame {f}: the coil is the same coil until the flick ({})",
            d.to_f32_for_render()
        );
    }
    let (p, q) = (pose_at(pair::POUNCE, 16), pose_at(pair::FEINT, 16));
    let d = tail(&p).sub(tail(&q)).abs();
    assert!(
        d.raw() > Fx::ratio(1, 12).raw(),
        "by frame sixteen the pounce's tail is up and over and the feint's is down ({})",
        d.to_f32_for_render()
    );
}

#[test]
fn the_rake_hold_is_drawn_when_the_second_hit_commits() {
    let mut holds = Vec::new();
    for seed in 0..12u32 {
        let mut w = duel(3);
        let m = w.monsters[0].as_mut().unwrap();
        m.brain.rng = (0x1234_5679 ^ seed.wrapping_mul(0x9E37_79B9)) | 1;
        throw(&mut w, pair::RAKE);
        // The swipe out: the hold is not drawn until it is over.
        let first = pair::SPECIES.attack(pair::RAKE);
        let rng_at_commit = cat(&w).brain.rng;
        for _ in 0..(first.startup + first.active) {
            w.advance([Input::aimed(Input::S, 0x8000), Input::default()]);
            assert_eq!(
                cat(&w).brain.rng,
                rng_at_commit,
                "nothing drawn during the first"
            );
        }
        let mut hold = 0;
        for _ in 0..80 {
            w.advance([Input::aimed(Input::S, 0x8000), Input::default()]);
            match cat(&w).doing {
                Doing::Startup {
                    kind: pair::COCK, ..
                } => hold += 1,
                Doing::Startup {
                    kind: pair::RAKE2, ..
                } => break,
                _ => {}
            }
        }
        assert!(
            (Knob::HoldMin.raw()..=Knob::HoldMax.raw() + 1).contains(&hold),
            "hold {hold}"
        );
        holds.push(hold);
    }
    holds.sort();
    holds.dedup();
    assert!(holds.len() > 3, "the hold varies: {holds:?}");
}

#[test]
fn nobody_stands_on_a_cat() {
    let w = duel(5);
    let m = cat(&w);
    // Put down on its back from above, a fighter is pushed off the side,
    // never held up.
    for x in [-1, 0, 1] {
        let above = base().add(V3::new(Fx::ratio(x * 4, 10), Fx::ratio(17, 10), Fx::ZERO));
        let c = m.resolve(
            above,
            sim::tuning::body_radius(),
            sim::tuning::body_height(),
        );
        assert!(c.landed.is_none(), "nothing on a cat is a floor");
        assert!(m.surface_under(above, sim::tuning::body_radius()).is_none());
    }
    // A fighter jumping onto it from the side comes down on the floor.
    let mut w = duel(3);
    w.players[0].pos = base().add(V3::new(Fx::ratio(15, 10), Fx::ZERO, Fx::ZERO));
    for f in 0..90 {
        let bits = if f < 30 {
            Input::SPACE | Input::W
        } else {
            Input::W
        };
        w.advance([Input::aimed(bits, 0x8000), Input::default()]);
        hold_still(&mut w);
        assert!(!me(&w).aboard(), "nobody mounts a cat");
    }
    assert!(
        me(&w).pos.y.raw() < Fx::ratio(1, 10).raw(),
        "and they end on the floor"
    );
}

#[test]
fn a_cat_that_cannot_see_you_does_not_sample_you() {
    // The Den's standing stone at (9.5, 9.5), between them.
    let mut w = hunt();
    w.monsters[1] = None;
    let at = V3::new(Fx::from_int(7), Fx::ZERO, Fx::from_int(7));
    let pin = |w: &mut World| {
        let m = w.monsters[0].as_mut().unwrap();
        m.brain.think_left = 9999;
        m.pos = at;
        m.speed = Fx::ZERO;
    };
    let m = w.monsters[0].as_mut().unwrap();
    m.brain.grace = 0;
    m.brain.seen = V3::ZERO;
    w.players[0].pos = V3::new(Fx::from_int(12), Fx::ZERO, Fx::from_int(12));
    for _ in 0..30 {
        w.advance([idle(), Input::default()]);
        pin(&mut w);
    }
    assert_eq!(cat(&w).brain.seen, V3::ZERO, "the stone hides you");
    assert!(fight::unseen(&cat(&w)) > 20);
    // Step out from behind it, and it sees you at its next glance.
    w.players[0].pos = V3::new(Fx::from_int(12), Fx::ZERO, Fx::from_int(4));
    for _ in 0..10 {
        w.advance([idle(), Input::default()]);
        pin(&mut w);
    }
    assert!(
        sim::math::wide_flat_dist(cat(&w).brain.seen, me(&w).pos).raw() < Fx::ONE.raw(),
        "in sight, it is sampled"
    );
}

#[test]
fn a_scarred_cat_does_not_see_you_on_its_blind_side() {
    let mut w = duel(5);
    let m = w.monsters[0].as_mut().unwrap();
    fight::scar_eye(m, 1);
    m.brain.seen = V3::ZERO;
    // Square off its right side: blind.
    w.players[0].pos = at(0, 5);
    for _ in 0..20 {
        w.advance([idle(), Input::default()]);
        hold_still(&mut w);
    }
    assert!(
        sim::math::wide_flat_dist(cat(&w).brain.seen, me(&w).pos).raw() > Fx::from_int(2).raw(),
        "on its blind side it does not sample you"
    );
    // Its good side: seen.
    w.players[0].pos = at(0, -5);
    for _ in 0..20 {
        w.advance([idle(), Input::default()]);
        hold_still(&mut w);
    }
    assert!(
        sim::math::wide_flat_dist(cat(&w).brain.seen, me(&w).pos).raw() < Fx::ONE.raw(),
        "on its good side it does"
    );
}

#[test]
fn three_hundred_and_fifty_into_a_head_scars_the_eye_on_that_side() {
    let mut w = duel(5);
    let mut m = cat(&w);
    let mut dealt = 0;
    while dealt < Knob::ScarDamage.raw() {
        dealt += m.take_hit(pair::HEAD, 40);
        m.doing = Doing::Prowl;
    }
    w.monsters[0] = Some(m);
    // The fighter is on its right.
    w.players[0].pos = at(1, 2);
    w.advance([idle(), Input::default()]);
    assert_eq!(fight::scar(&cat(&w)), 1);
}

#[test]
#[ignore]
fn debug_pounce() {
    let toward = Input::aimed(Input::SHIFT | Input::W, 0x8000);
    let mut w = duel(7);
    throw(&mut w, pair::POUNCE);
    for f in 0..96u32 {
        let input = if (28..30).contains(&f) { toward } else { idle() };
        w.advance([input, Input::default()]);
        let m = cat(&w);
        let p = me(&w);
        let hv = m.hit_volume();
        println!(
            "{f:3} {:?} cat ({:.2},{:.2},{:.2}) aim ({:.2},{:.2}) me ({:.2},{:.2}) {:?} hp {} vol {:?}",
            m.doing,
            m.pos.x.to_f32_for_render(),
            m.pos.y.to_f32_for_render(),
            m.pos.z.to_f32_for_render(),
            m.aimed_at().x.to_f32_for_render(),
            m.aimed_at().z.to_f32_for_render(),
            p.pos.x.to_f32_for_render(),
            p.pos.z.to_f32_for_render(),
            p.action,
            p.health,
            hv.map(|(a, r, _, _)| (a.x.to_f32_for_render(), a.z.to_f32_for_render(), r.to_f32_for_render()))
        );
    }
}

#[test]
#[ignore]
fn debug_flick() {
    for f in 0..24u16 {
        let pose_at = |kind: u8| {
            let mut m = Monster::new(SpeciesId::PAIR);
            let a = pair::SPECIES.attack(kind);
            m.doing = Doing::Startup { kind, left: a.startup - f };
            m.pose()
        };
        let (p, q) = (pose_at(pair::POUNCE), pose_at(pair::FEINT));
        println!("{f} {:?} {:?}", (1..4).map(|b| p.bone[pair::bones::TAIL1 + b - 1].x.to_f32_for_render()).collect::<Vec<_>>(),
          (1..4).map(|b| q.bone[pair::bones::TAIL1 + b - 1].x.to_f32_for_render()).collect::<Vec<_>>());
    }
}
