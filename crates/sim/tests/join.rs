//! **Joining a fight in progress, and starting the match together**: the
//! world's half of a room people come and go from (`docs/design/versus.md`,
//! "A room you come and go from").
//!
//! - "Start match" is a flag each seat flips on the wire
//!   (`Travel::START`); the frame every seat in the fight has it -- two at
//!   least -- the world starts the match itself, so every machine starts it
//!   on the same frame.
//! - A newcomer takes a free seat on the smaller side (`World::open_seat`),
//!   and stands where a fresh start of the same fight would stand them
//!   (`World::seat_join`), with nobody else touched.

use sim::class::Class;
use sim::input::Travel;
use sim::state::MAX_PLAYERS;
use sim::{Input, World};

fn wire(n: usize, ready: &[usize]) -> [Input; MAX_PLAYERS] {
    let mut w = [Input::default(); MAX_PLAYERS];
    for &i in ready {
        if i < n {
            w[i] = Input::default().travelling(Travel::START);
        }
    }
    w
}

/// Two in the town, as a room starts.
fn hub(n: u8) -> World {
    sim::replay::Start::parse(&format!(
        "p1=champion p2=elementalist arena=hearth temper=0 seats={n}"
    ))
    .expect("start")
    .world()
}

#[test]
fn the_match_starts_the_frame_everybody_is_ready_and_not_before() {
    let mut w = hub(2);
    w.advance(wire(2, &[0]));
    assert!(w.starting.has(0) && !w.starting.has(1));
    assert!(
        sim::versus::of(w.arena).is_none(),
        "one ready is not a match"
    );
    for _ in 0..30 {
        w.advance(wire(2, &[]));
    }
    assert!(sim::versus::of(w.arena).is_none(), "still waiting for two");
    let frame = w.frame;
    w.advance(wire(2, &[1]));
    assert_eq!(
        w.frame,
        frame + 1,
        "the frame keeps counting through the start"
    );
    assert_eq!(
        w.arena,
        sim::versus::STAGES[0].arena,
        "from the town, the first duel stage"
    );
    assert!(!w.starting.any(), "a fresh match has nobody ready");
    assert!(w.is_present(0) && w.is_present(1));
}

#[test]
fn pressing_again_takes_it_back() {
    let mut w = hub(2);
    w.advance(wire(2, &[0]));
    w.advance(wire(2, &[0]));
    assert!(!w.starting.any());
    w.advance(wire(2, &[1]));
    assert!(
        sim::versus::of(w.arena).is_none(),
        "seat one took theirs back"
    );
}

#[test]
fn on_a_stage_ready_is_a_fresh_round_of_it() {
    let stage = sim::versus::STAGES[2].arena;
    let mut w = World::versus_in([Class::Champion, Class::Champion], stage).seated(2);
    for _ in 0..90 {
        w.advance(wire(2, &[]));
    }
    w.advance(wire(2, &[0, 1]));
    assert_eq!(w.arena, stage);
    assert!(!w.starting.any());
}

#[test]
fn a_seat_that_has_left_neither_holds_the_start_up_nor_counts() {
    let mut w = hub(3);
    w.advance(wire(3, &[0, 1]));
    assert!(
        sim::versus::of(w.arena).is_none(),
        "the third is in the fight and not ready"
    );
    // The third leaves: the session writes it into their input.
    let mut gone = wire(3, &[]);
    gone[2] = Input::default().travelling(Travel::LEFT);
    w.advance(gone);
    assert_eq!(
        w.arena,
        sim::versus::STAGES[0].arena,
        "the two who are left were both ready"
    );
}

#[test]
fn a_newcomer_takes_the_smaller_side() {
    let w = hub(2);
    // One a side: the coin decides, and each side's lowest free seat.
    assert_eq!(w.open_seat(false), Some(2));
    assert_eq!(w.open_seat(true), Some(3));
    let mut three = w.clone();
    three.seat_join(2, Class::Champion);
    // Two on side nought, one on side one: side one, whatever the coin.
    assert_eq!(three.open_seat(false), Some(3));
    assert_eq!(three.open_seat(true), Some(3));
    let mut four = three.clone();
    four.seat_join(3, Class::Champion);
    assert_eq!(four.open_seat(false), None, "four is everybody");
}

#[test]
fn a_seat_left_empty_is_taken_again() {
    let mut w = hub(4);
    let mut gone = wire(4, &[]);
    gone[1] = Input::default().travelling(Travel::LEFT);
    w.advance(gone);
    assert!(!w.is_present(1));
    assert_eq!(w.open_seat(false), Some(1), "side one is a fighter short");
}

#[test]
fn a_newcomer_stands_where_a_fresh_start_would_and_nobody_else_moves() {
    let mut w = hub(2);
    for f in 0..120u32 {
        let mut i = [Input::default(); MAX_PLAYERS];
        i[0] = Input::aimed(Input::W, (f * 300) as u16);
        i[1] = Input::aimed(Input::W | Input::LEFT, 0);
        w.advance(i);
    }
    let before = w.clone();
    let mut a = w.clone();
    a.seat_join(2, Class::Elementalist);
    assert!(a.is_present(2));
    assert_eq!(a.players[2].class, Class::Elementalist);
    assert_eq!(a.players[2].health, a.players[2].full_health());
    for i in 0..2 {
        assert_eq!(a.players[i].pos, before.players[i].pos, "fighter {i} moved");
    }
    let mut classes = w.players.map(|p| p.class);
    classes[2] = Class::Elementalist;
    let mut grown = w.clone();
    grown.seats = 3;
    let fresh = grown.restarted(classes);
    assert_eq!(a.players[2].pos, fresh.players[2].pos);
    // Every machine doing it to the same world gets the same world.
    let mut b = w.clone();
    b.seat_join(2, Class::Elementalist);
    assert_eq!(a.checksum(), b.checksum());
    // And it plays on in step.
    for _ in 0..60 {
        a.advance([Input::default(); MAX_PLAYERS]);
        b.advance([Input::default(); MAX_PLAYERS]);
    }
    assert_eq!(a.checksum(), b.checksum());
    assert!(a.is_present(2));
}

#[test]
fn the_host_admits_a_newcomer_on_the_wire() {
    // A room for four, two in it: the empty seats are said to have left.
    let mut w = hub(4);
    let mut wire = [Input::default(); MAX_PLAYERS];
    wire[2] = Input::default().travelling(Travel::LEFT);
    wire[3] = Input::default().travelling(Travel::LEFT);
    w.advance(wire);
    assert!(!w.is_present(2) && !w.is_present(3));
    // The host seats a newcomer in seat three; seat two is still nobody's.
    let mut admit = wire;
    admit[0] = Input::default().travelling(Travel::admit(3, Class::DualMage));
    w.advance(admit);
    assert!(w.is_present(3), "seated on the frame it arrives");
    assert_eq!(w.players[3].class, Class::DualMage);
    assert!(!w.is_present(2));
    // From the next frame the newcomer has an input of their own.
    let mut next = [Input::default(); MAX_PLAYERS];
    next[2] = Input::default().travelling(Travel::LEFT);
    w.advance(next);
    assert!(w.is_present(3));
    assert_eq!(
        Travel::admit(3, Class::DualMage).destination(),
        Some(sim::input::Destination::Admit {
            seat: 3,
            class: Class::DualMage
        })
    );
    assert!(
        matches!(
            Travel::class(3, Class::DualMage).destination(),
            Some(sim::input::Destination::Class { .. })
        ),
        "a class change is still a class change"
    );
}
