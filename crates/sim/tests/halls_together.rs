//! **The tutorial halls are a shared space** (2026-10-25). Every hall is a
//! place on the valley's map, and a trip into one takes only whoever asked:
//!
//! - one player going to a hall leaves the other where they were;
//! - two who each go to the same hall are in it together, with one dummy
//!   between them, and neither can hurt the other;
//! - each one's checklist is their own, ticked by what they did;
//! - two in different halls have a dummy each;
//! - V takes only the one who pressed it back to the town.

use sim::input::Travel;
use sim::state::MAX_PLAYERS;
use sim::tutorial::{self, Progress};
use sim::{Class, Fx, Input, V3, World};

/// Two people in Hearth's square, the valley a run starts in.
fn town(classes: [Class; 2]) -> World {
    World::arrive(
        classes,
        sim::valley::Journey::default(),
        sim::valley::START,
        None,
        2,
    )
}

/// One frame, each seat's input; both tallies fed, each for its own seat.
fn frame(w: &mut World, inputs: [Input; 2], coaches: &mut [Progress; 2]) {
    let before = w.clone();
    w.advance(inputs);
    for (me, coach) in coaches.iter_mut().enumerate() {
        coach.observe(&before, w, me);
    }
}

fn idle(w: &mut World, frames: u32, coaches: &mut [Progress; 2]) {
    for _ in 0..frames {
        frame(w, [Input::default(); 2], coaches);
    }
}

/// Stand seat `i` in room `k` of the hall they are in, `back` metres short
/// of the dummy's mark and facing down the hall.
fn stand_in(w: &mut World, i: usize, k: usize, back: i32) {
    let (t, _) = w.hall_of(i).expect("in a hall");
    let hall = sim::atlas::valley()
        .placed(t.arena)
        .expect("on the map")
        .at
        .sub(w.map_origin());
    let mark = sim::arena::hall::dummy_mark(t.rooms, k);
    let p = &mut w.players[i];
    p.pos = mark
        .at
        .add(hall)
        .sub(V3::new(Fx::from_int(back), Fx::ZERO, Fx::ZERO));
    p.facing = V3::new(Fx::ONE, Fx::ZERO, Fx::ZERO);
    p.vel = V3::ZERO;
}

/// Seat `i`'s buttons with the crosshair on its hall's dummy.
fn at_dummy(w: &World, i: usize, bits: u16) -> Input {
    let d = w.dummy_for(i).expect("a dummy in the hall");
    let me = &w.players[i];
    let at = w.players[d].pos.add(V3::new(Fx::ZERO, Fx::ONE, Fx::ZERO));
    let to = at.sub(me.pos);
    let aim = (sim::math::atan2_turns(to.z, to.x).raw() & 0xFFFF) as u16;
    let pitch = sim::aim::look_onto(me.pos, aim, me.aloft, me.clear, at);
    Input::looking_at(bits, aim, pitch)
}

fn hall_of(w: &World, i: usize) -> Option<sim::arena::ArenaId> {
    w.hall_of(i).map(|(t, _)| t.arena)
}

#[test]
fn a_trip_into_a_hall_takes_only_whoever_asked() {
    let mut w = town([Class::Champion, Class::Champion]);
    let mut coaches = [Progress::default(), Progress::default()];
    idle(&mut w, 5, &mut coaches);
    let stayed = w.players[0].pos.add(w.map_origin());
    let hall = tutorial::for_class(Class::Champion).unwrap().arena;
    frame(
        &mut w,
        [
            Input::default(),
            Input::default().travelling(Travel::arena(hall)),
        ],
        &mut coaches,
    );
    idle(&mut w, 30, &mut coaches);
    assert_eq!(hall_of(&w, 1), Some(hall), "player two went");
    assert_eq!(hall_of(&w, 0), None, "player one stayed");
    assert_eq!(
        w.players[0].pos.add(w.map_origin()),
        stayed,
        "and did not move"
    );
    assert_eq!(
        w.arena,
        sim::valley::START,
        "the frame stays with player one"
    );
    // A dummy for the hall, in a seat nobody plays.
    let d = w.dummy_for(1).expect("the hall has its dummy");
    assert!(d >= 2 && w.is_present(d));
    assert!(w.foes(1, d) && w.foes(d, 1));
    assert!(!w.foes(0, d) && !w.foes(0, 1), "nobody else is fair game");

    // **Player two's checklist is player two's.** A sword string into the
    // dummy ticks theirs and not player one's.
    stand_in(&mut w, 1, 1, 2);
    idle(&mut w, 30, &mut coaches);
    for f in 0..100 {
        let bits = if f < 2 { Input::LEFT } else { 0 };
        let input = at_dummy(&w, 1, bits);
        frame(&mut w, [Input::default(), input], &mut coaches);
    }
    assert_eq!(
        coaches[1].times(Class::Champion, 1, 0),
        1,
        "player two's tick"
    );
    assert_eq!(
        coaches[0].times(Class::Champion, 1, 0),
        0,
        "not player one's"
    );

    // **V takes only the one who pressed it back.**
    frame(
        &mut w,
        [
            Input::default(),
            Input::default().travelling(Travel::VALLEY),
        ],
        &mut coaches,
    );
    idle(&mut w, 5, &mut coaches);
    assert_eq!(hall_of(&w, 1), None, "player two is back");
    assert!(w.dummies.iter().all(|&d| d == 0), "and the hall is empty");
    assert_eq!(
        w.players[0].pos.add(w.map_origin()),
        stayed,
        "player one never moved"
    );
}

#[test]
fn two_in_one_hall_share_its_dummy_and_keep_their_own_lists() {
    let mut w = town([Class::Champion, Class::Champion]);
    let mut coaches = [Progress::default(), Progress::default()];
    let hall = tutorial::for_class(Class::Champion).unwrap().arena;
    let go = Input::default().travelling(Travel::arena(hall));
    frame(&mut w, [go, Input::default()], &mut coaches);
    idle(&mut w, 10, &mut coaches);
    frame(&mut w, [Input::default(), go], &mut coaches);
    idle(&mut w, 30, &mut coaches);
    assert_eq!(hall_of(&w, 0), Some(hall));
    assert_eq!(hall_of(&w, 1), Some(hall));
    let d = w.dummy_for(0).expect("a dummy");
    assert_eq!(w.dummy_for(1), Some(d), "one dummy between them");
    assert_eq!(
        w.dummies.iter().filter(|&&k| k != 0).count(),
        1,
        "and only one"
    );
    assert!(
        !w.foes(0, 1) && !w.foes(1, 0),
        "they cannot hurt each other"
    );

    // Player one strikes the dummy; only player one's box ticks.
    stand_in(&mut w, 0, 1, 2);
    stand_in(&mut w, 1, 1, 6);
    idle(&mut w, 30, &mut coaches);
    for f in 0..100 {
        let bits = if f < 2 { Input::LEFT } else { 0 };
        let input = at_dummy(&w, 0, bits);
        frame(&mut w, [input, Input::default()], &mut coaches);
    }
    assert_eq!(coaches[0].times(Class::Champion, 1, 0), 1);
    assert_eq!(coaches[1].times(Class::Champion, 1, 0), 0);
    // Then player two does, from where player one stood; both have it
    // now, once each.
    stand_in(&mut w, 0, 1, 8);
    stand_in(&mut w, 1, 1, 2);
    idle(&mut w, 30, &mut coaches);
    for f in 0..100 {
        let bits = if f < 2 { Input::LEFT } else { 0 };
        let input = at_dummy(&w, 1, bits);
        frame(&mut w, [Input::default(), input], &mut coaches);
    }
    assert_eq!(coaches[0].times(Class::Champion, 1, 0), 1);
    assert_eq!(coaches[1].times(Class::Champion, 1, 0), 1);
    let full = |w: &World, i: usize| w.players[i].health == w.players[i].full_health();
    assert!(full(&w, 0) && full(&w, 1), "neither was hurt by the other");
}

#[test]
fn two_halls_have_a_dummy_each() {
    let mut w = town([Class::ShadowReaver, Class::Champion]);
    let mut coaches = [Progress::default(), Progress::default()];
    let reaver = tutorial::for_class(Class::ShadowReaver).unwrap().arena;
    let champion = tutorial::for_class(Class::Champion).unwrap().arena;
    frame(
        &mut w,
        [
            Input::default().travelling(Travel::arena(reaver)),
            Input::default().travelling(Travel::arena(champion)),
        ],
        &mut coaches,
    );
    idle(&mut w, 30, &mut coaches);
    assert_eq!(hall_of(&w, 0), Some(reaver));
    assert_eq!(hall_of(&w, 1), Some(champion));
    let (a, b) = (w.dummy_for(0).unwrap(), w.dummy_for(1).unwrap());
    assert_ne!(a, b);
    assert!(a >= 2 && b >= 2 && a < MAX_PLAYERS && b < MAX_PLAYERS);
    assert!(
        !w.foes(0, b) && !w.foes(1, a),
        "each hall's dummy is its own"
    );
    // Each dummy stands in its own hall.
    for (i, d) in [(0, a), (1, b)] {
        let mine = w.players[i].pos.add(w.map_origin());
        let its = w.players[d].pos.add(w.map_origin());
        let place = sim::atlas::valley().place_for(its).map(|p| p.arena);
        assert_eq!(place, hall_of(&w, i), "seat {i}'s dummy is in their hall");
        assert!(mine.sub(its).flat_len().raw() < Fx::from_int(30).raw());
    }
}
