//! Changing class is a request on the wire, so it works against a person.
//!
//! Tab and the class pickers used to rebuild the world on the machine that
//! pressed them, which against a person is a desync, so a match refused
//! them and the two players were stuck with the classes the link named. Now a
//! class change is `Travel::class` in the player's input: predicted, confirmed
//! and rolled back like a button, and acted on by `World::advance` on the same
//! frame on both machines.
//!
//! **And it touches nobody else** (2026-10-22): it used to restart the whole
//! fight, which against people is everybody's round stopping for one player.
//! The fighter who asked is recast where they stand. A versus world that
//! drafts (`World::drafting`, what the game plays) chooses classes in a pick
//! before each round and locks them for the round itself.

use sim::World;
use sim::class::{ALL_CLASSES, Class};
use sim::input::{Destination, Input, Travel};
use sim::species::SpeciesId;
use sim::state::Phase;

#[test]
fn every_class_for_every_seat_survives_the_travel_byte() {
    for seat in 0..sim::state::MAX_PLAYERS {
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
    for b in 0..=u16::MAX {
        match Travel(b).destination() {
            Some(Destination::Class { seat, class }) => {
                assert_eq!(Travel::class(seat, class), Travel(b), "word {b:#06x}")
            }
            Some(Destination::Ready { seat }) => {
                assert_eq!(Travel::ready(seat), Travel(b), "word {b:#06x}")
            }
            _ => {}
        }
    }
    // An item, fed or granted: the item in the high byte of a credit's.
    for b in 0..=u16::MAX {
        match Travel(b).destination() {
            Some(Destination::Feed(item)) => {
                assert_eq!(Travel::feed(item), Travel(b), "word {b:#06x}")
            }
            Some(Destination::Grant(item)) => {
                assert_eq!(Travel::grant(item), Travel(b), "word {b:#06x}")
            }
            _ => {}
        }
    }
    // A cairn: seven, with its number in the high byte -- every number.
    for b in 0..=u16::MAX {
        if let Some(Destination::Cairn(k)) = Travel(b).destination() {
            assert_eq!(Travel::cairn(k), Travel(b), "word {b:#06x}");
        }
    }
    for k in 0..=u8::MAX {
        assert_eq!(Travel::cairn(k).destination(), Some(Destination::Cairn(k)));
    }
    // An arena: its id's high bits in the high byte (2026-10-23, for the
    // tutorial halls past sixty-three) -- every word that reads as one is
    // the word that arena writes.
    for b in 0..=u16::MAX {
        if let Some(Destination::Arena(id)) = Travel(b).destination() {
            assert_eq!(Travel::arena(id), Travel(b), "word {b:#06x}");
        }
    }
    // A high byte on anything else asks for nothing: a byte that meant a
    // trip still means exactly that trip, and nothing else.
    for b in 0..=255u16 {
        if Travel(b).destination().is_some_and(|d| {
            !matches!(
                d,
                Destination::Class { .. }
                    | Destination::Ready { .. }
                    | Destination::Feed(_)
                    | Destination::Credit(_)
                    | Destination::Cairn(_)
                    | Destination::Arena(_)
            )
        }) {
            assert_eq!(
                Travel(b | 0x100).destination(),
                None,
                "word {:#06x}",
                b | 0x100
            );
        }
    }
    // A seat far past four survives it.
    assert_eq!(
        Travel::class(1000, Class::Champion).destination(),
        Some(Destination::Class {
            seat: 1000,
            class: Class::Champion
        })
    );
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
fn every_seat_ready_survives_the_travel_byte() {
    for seat in [0, 1, 2, 3, 1000] {
        assert_eq!(
            Travel::ready(seat).destination(),
            Some(Destination::Ready { seat })
        );
    }
}

#[test]
fn a_class_change_recasts_that_seat_where_it_stands() {
    let mut w = World::with_classes([Class::Bulwark, Class::Bulwark]);
    for _ in 0..30 {
        w.advance([Input::new(Input::W), Input::new(Input::W)]);
    }
    let frame = w.frame;
    let before = w.clone();
    w.advance([Input::new(Input::W), ask(Travel::class(1, Class::Champion))]);
    assert_eq!(w.players[0].class, Class::Bulwark);
    assert_eq!(w.players[1].class, Class::Champion);
    assert_eq!(
        w.frame,
        frame + 1,
        "the rollback session owns the frame count"
    );
    let fresh = World::with_classes([Class::Bulwark, Class::Champion]);
    assert_ne!(
        w.players[0].pos, fresh.players[0].pos,
        "player one's fight went on: nobody was put back on a mark"
    );
    assert_ne!(
        w.players[0].pos, before.players[0].pos,
        "and player one walked this frame as on any other"
    );
    assert_eq!(
        w.players[1].pos.x, before.players[1].pos.x,
        "player two is recast on the spot they stood on"
    );
}

#[test]
fn a_class_change_is_not_a_heal() {
    let mut w = World::with_classes([Class::Bulwark, Class::Bulwark]);
    let full = w.players[1].full_health();
    w.players[1].health = full / 2;
    w.advance([Input::default(), ask(Travel::class(1, Class::Champion))]);
    let now = w.players[1];
    let share = now.health as i64 * 100 / now.full_health() as i64;
    assert!(
        (49..=51).contains(&share),
        "half a bar stays half a bar: {share}%"
    );
}

/// A drafting duel, as the game plays one.
fn drafted() -> World {
    World::with_classes([Class::Champion, Class::Champion]).drafting(true)
}

#[test]
fn a_drafted_versus_round_opens_with_its_pick() {
    let w = drafted();
    assert!(matches!(w.phase, Phase::Picking { .. }));
    assert!(!w.pvp(), "nobody can hurt anybody while choosing");
    assert!(!w.classes_locked());
    // A hunt, a course and a world built bare do not.
    assert!(matches!(
        World::hunt([Class::Champion, Class::Champion])
            .drafting(true)
            .phase,
        Phase::Fighting
    ));
    assert!(matches!(
        World::with_classes([Class::Champion, Class::Champion]).phase,
        Phase::Fighting
    ));
}

#[test]
fn the_pick_ends_on_its_clock_and_locks_the_classes() {
    let mut w = drafted();
    w.advance([ask(Travel::class(0, Class::DualMage)), Input::default()]);
    assert_eq!(w.players[0].class, Class::DualMage, "heard in the pick");
    for _ in 0..sim::tuning::class_pick() + 1 {
        w.advance([Input::default(), Input::default()]);
    }
    assert_eq!(w.phase, Phase::Fighting, "the round is on");
    assert!(w.pvp() && w.classes_locked());
    w.advance([ask(Travel::class(0, Class::Elementalist)), Input::default()]);
    assert_eq!(
        w.players[0].class,
        Class::DualMage,
        "locked for the round: the request is not heard"
    );
}

#[test]
fn the_pick_ends_early_once_every_seat_is_ready() {
    let mut w = drafted();
    w.advance([ask(Travel::ready(0)), Input::default()]);
    assert!(
        matches!(w.phase, Phase::Picking { .. }),
        "one of two is not everybody"
    );
    w.advance([Input::default(), ask(Travel::ready(1))]);
    w.advance([Input::default(), Input::default()]);
    assert_eq!(w.phase, Phase::Fighting);
}

#[test]
fn the_next_round_opens_with_a_pick_again() {
    let mut w = drafted();
    w.advance([ask(Travel::ready(0)), ask(Travel::ready(1))]);
    w.advance([Input::default(), Input::default()]);
    assert_eq!(w.phase, Phase::Fighting);
    w.players[1].health = 0;
    w.advance([Input::default(), Input::default()]);
    assert!(matches!(w.phase, Phase::RoundOver { winner: 0, .. }));
    for _ in 0..sim::tuning::round_over_frames() + 2 {
        w.advance([Input::default(), Input::default()]);
    }
    assert!(matches!(w.phase, Phase::Picking { .. }));
    assert_eq!(w.players[0].rounds_won, 1, "the score is kept");
    w.advance([Input::default(), ask(Travel::class(1, Class::ShadowReaver))]);
    assert_eq!(w.players[1].class, Class::ShadowReaver);
}

#[test]
fn a_trip_carries_the_draft() {
    let mut w = drafted();
    w.advance([ask(Travel::hunt(SpeciesId::RIDGEBACK)), Input::default()]);
    assert!(w.draft && w.hunting());
    assert_eq!(w.phase, Phase::Fighting, "a hunt has no pick");
    w.advance([ask(Travel::VERSUS), Input::default()]);
    assert!(matches!(w.phase, Phase::Picking { .. }));
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
    let mut w = World::with_classes([Class::Bulwark, Class::Bulwark]).drafting(true);
    w.advance([ask(Travel::hunt(SpeciesId::RIDGEBACK)), Input::default()]);
    let start = w.players[0].pos;
    for _ in 0..30 {
        w.advance([Input::new(Input::W), Input::default()]);
    }
    let walked = w.players[0].pos;
    assert_ne!(walked, start);
    w.advance([ask(Travel::class(0, Class::ShadowReaver)), Input::default()]);
    assert!(w.hunting(), "a recast is the same fight, not versus");
    assert_eq!(w.players[0].class, Class::ShadowReaver, "never locked");
    assert_ne!(
        w.players[0].pos, start,
        "nobody was put back at the hunt's start"
    );
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
