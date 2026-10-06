//! Changing class is a trip on the wire, so it works against a person.
//!
//! Tab and the class pickers used to rebuild the world on the machine that
//! pressed them, which against a person is a desync, so a match refused
//! them and the two players were stuck with the classes the link named. Now a
//! class change is `Travel::class` in the player's input: predicted, confirmed
//! and rolled back like a button, and acted on by `World::advance` on the same
//! frame on both machines.

use sim::World;
use sim::class::{ALL_CLASSES, Class};
use sim::input::{Destination, Input, Travel};
use sim::species::SpeciesId;

#[test]
fn every_class_for_either_seat_survives_the_travel_byte() {
    for seat in 0..2 {
        for class in ALL_CLASSES {
            assert_eq!(
                Travel::class(seat, class).destination(),
                Some(Destination::Class { seat, class }),
                "seat {seat} as {class:?}"
            );
        }
    }
}

#[test]
fn no_two_kinds_of_trip_share_a_byte() {
    // Every byte means at most one thing, and the bytes that already meant
    // something -- a hunt at a temper, an arena -- still mean it.
    for b in 0..=255u8 {
        let Some(Destination::Class { seat, class }) = Travel(b).destination() else {
            continue;
        };
        assert_eq!(Travel::class(seat, class), Travel(b), "byte {b:#04x}");
    }
    assert!(matches!(
        Travel::hunt(SpeciesId::RIDGEBACK).destination(),
        Some(Destination::Hunt(..))
    ));
    assert_eq!(Travel::VERSUS.destination(), Some(Destination::Versus));
}

fn ask(travel: Travel) -> Input {
    Input::default().travelling(travel)
}

#[test]
fn a_class_change_restarts_the_fight_with_that_seat_recast() {
    let mut w = World::with_classes([Class::Bulwark, Class::Bulwark]);
    for _ in 0..30 {
        w.advance([Input::new(Input::W), Input::default()]);
    }
    let frame = w.frame;
    w.advance([Input::default(), ask(Travel::class(1, Class::Champion))]);
    assert_eq!(w.players[0].class, Class::Bulwark);
    assert_eq!(w.players[1].class, Class::Champion);
    assert_eq!(
        w.frame,
        frame + 1,
        "the rollback session owns the frame count"
    );
    let fresh = World::with_classes([Class::Bulwark, Class::Champion]);
    assert_eq!(w.players[0].pos, fresh.players[0].pos, "a fresh fight");
}

#[test]
fn both_players_may_change_class_on_one_frame() {
    let mut w = World::with_classes([Class::Bulwark, Class::Bulwark]);
    w.advance([
        ask(Travel::class(0, Class::Elementalist)),
        ask(Travel::class(1, Class::DualMage)),
    ]);
    assert_eq!(w.players[0].class, Class::Elementalist);
    assert_eq!(w.players[1].class, Class::DualMage);
}

#[test]
fn a_trip_on_the_same_frame_keeps_the_new_class() {
    let mut w = World::with_classes([Class::Bulwark, Class::Bulwark]);
    w.advance([
        ask(Travel::hunt(SpeciesId::RIDGEBACK)),
        ask(Travel::class(1, Class::BloodMage)),
    ]);
    assert!(w.hunting());
    assert_eq!(w.players[1].class, Class::BloodMage);
}

#[test]
fn a_class_change_mid_hunt_stays_in_the_hunt() {
    let mut w = World::with_classes([Class::Bulwark, Class::Bulwark]);
    w.advance([ask(Travel::hunt(SpeciesId::RIDGEBACK)), Input::default()]);
    w.advance([ask(Travel::class(0, Class::ShadowReaver)), Input::default()]);
    assert!(w.hunting(), "a recast restarts the same fight, not versus");
    assert_eq!(w.players[0].class, Class::ShadowReaver);
}

// Pause, step and restart: the rest of what one keyboard does to the fight,
// on the wire for the same reason.

#[test]
fn a_paused_world_changes_nothing_but_the_frame() {
    let mut w = World::new();
    for _ in 0..10 {
        w.advance([Input::new(Input::W), Input::default()]);
    }
    w.advance([Input::default(), ask(Travel::PAUSE)]);
    assert!(w.paused);
    let held = w.clone();
    for _ in 0..20 {
        w.advance([Input::new(Input::W | Input::LEFT), Input::default()]);
    }
    assert_eq!(
        w.frame,
        held.frame + 20,
        "the session's frame count goes on"
    );
    assert_eq!(w.players, held.players, "nothing else does");
    w.advance([ask(Travel::PAUSE), Input::default()]);
    assert!(!w.paused, "either side unpauses");
    w.advance([Input::new(Input::W), Input::default()]);
    assert_ne!(w.players[0].pos, held.players[0].pos);
}

#[test]
fn a_step_pauses_and_plays_exactly_its_own_frame() {
    let mut w = World::new();
    w.advance([ask(Travel::STEP).with(Input::W), Input::default()]);
    assert!(w.paused);
    let after_one = w.players[0].pos;
    assert_ne!(
        after_one,
        World::new().players[0].pos,
        "its frame was played"
    );
    w.advance([Input::new(Input::W), Input::default()]);
    assert_eq!(w.players[0].pos, after_one, "and only its frame");
    w.advance([ask(Travel::STEP).with(Input::W), Input::default()]);
    assert_ne!(w.players[0].pos, after_one);
    assert!(w.paused);
}

#[test]
fn a_restart_keeps_the_classes_and_the_pause() {
    let mut w = World::with_classes([Class::Champion, Class::DualMage]);
    for _ in 0..30 {
        w.advance([Input::new(Input::W), Input::default()]);
    }
    w.advance([ask(Travel::PAUSE), Input::default()]);
    w.advance([Input::default(), ask(Travel::RESTART)]);
    let fresh = World::with_classes([Class::Champion, Class::DualMage]);
    assert_eq!(w.players[0].pos, fresh.players[0].pos);
    assert_eq!(w.players[1].class, Class::DualMage);
    assert!(
        w.paused,
        "a restart while paused stays paused, as it does at one keyboard"
    );
}
