//! Tempers (world W2): a beaten creature fought again, cleverer rather than
//! tougher. See `sim::temper` and `docs/design/world.md` §4.
//!
//! What has to stay true: a temper changes the four brain knobs and nothing
//! else; the same tempered hunt plays out the same every time; the temper is in
//! the hunt's initial state and travels with it; and versus never sees one.

use sim::input::{Destination, Travel};
use sim::species::{self, SpeciesId};
use sim::state::{MAX_PLAYERS, Phase, QUARRY};
use sim::temper::{self, HIGHEST, TEMPERS};
use sim::{Class, Input, World};

fn classes() -> [Class; MAX_PLAYERS] {
    [Class::Bulwark; MAX_PLAYERS]
}

/// Walk at it and swing: enough to make the creature think.
fn pressing(frame: usize) -> [Input; MAX_PLAYERS] {
    let mut buttons = Input::W;
    if frame % 40 < 3 {
        buttons |= Input::LEFT;
    }
    if frame % 97 < 2 {
        buttons |= Input::SHIFT | Input::A;
    }
    [Input::new(buttons), Input::new(Input::D)]
}

fn run(mut w: World, frames: usize) -> World {
    for f in 0..frames {
        w.advance(pressing(f));
    }
    w
}

#[test]
fn each_temper_is_cleverer_than_the_one_below_it() {
    // The shape world.md §4 promises: the glance shortens, the lead lengthens,
    // the decisiveness rises, and the thresholds fall less far. Strictly for
    // the glance and the lead, so a temper is never a no-op on a creature with
    // any glance or lead at all.
    let w = World::hunt(classes());
    let mut last = *w.monster().unwrap();
    assert_eq!(last.temper, 0);
    for n in 1..TEMPERS {
        let m = *World::hunt(classes()).tempered(n).monster().unwrap();
        assert_eq!(m.temper, n);
        assert!(
            m.glance_frames() < last.glance_frames(),
            "temper {n} glance"
        );
        assert!(m.lead().raw() > last.lead().raw(), "temper {n} lead");
        assert!(
            m.decisiveness() >= last.decisiveness(),
            "temper {n} decisiveness"
        );
        assert!(
            m.strain_desperation() < last.strain_desperation(),
            "temper {n} desperation"
        );
        // And nothing it is made of: the same health, the same hide.
        assert_eq!(m.health, last.health);
        assert_eq!(m.breaks, last.breaks);
        last = m;
    }
}

#[test]
fn temper_zero_is_the_creature_as_tuned() {
    let as_tuned = *World::hunt(classes()).monster().unwrap();
    let sp = as_tuned.sp();
    assert_eq!(as_tuned.glance_frames(), sp.glance_frames());
    assert_eq!(as_tuned.lead(), sp.lead());
    assert_eq!(as_tuned.decisiveness(), sp.decisiveness());
    assert_eq!(as_tuned.strain_desperation(), sp.strain_desperation());
    assert_eq!(temper::of(0), temper::AS_TUNED);
    // And a hunt built at temper zero is the hunt there always was, bit for
    // bit: the byte is not hashed while it is zero.
    assert_eq!(
        World::hunt(classes()).tempered(0).checksum(),
        World::hunt(classes()).checksum()
    );
}

#[test]
fn a_pack_is_tempered_through_the_same_three_numbers() {
    // Pack-only species (critters) have no monster; the pack's own glance,
    // lead and thinking cadence take the temper instead.
    let plain = World::hunt_of(classes(), SpeciesId::GNATS);
    let hard = World::hunt_of(classes(), SpeciesId::GNATS).tempered(HIGHEST);
    assert!(plain.monster().is_none());
    let (p, h) = (plain.pack.unwrap(), hard.pack.unwrap());
    assert_eq!((p.temper, h.temper), (0, HIGHEST));
    assert_eq!(hard.temper(), HIGHEST);
    assert!(h.glance_frames() <= p.glance_frames());
    assert!(h.lead_frames() >= p.lead_frames());
    assert!(h.think_every() <= p.think_every());
    assert!(
        h.glance_frames() < p.glance_frames() || h.lead_frames() > p.lead_frames(),
        "the highest temper changed nothing about the dev pack"
    );
}

#[test]
fn a_tempered_hunt_plays_differently_and_the_same_every_time() {
    let tempered = || run(World::hunt(classes()).tempered(HIGHEST), 900);
    let a = tempered();
    let b = tempered();
    assert_eq!(
        a.checksum(),
        b.checksum(),
        "a tempered hunt is not deterministic"
    );
    assert_eq!(a.temper(), HIGHEST);
    let plain = run(World::hunt(classes()), 900);
    assert_ne!(
        plain.checksum(),
        a.checksum(),
        "the highest temper played exactly as the creature as tuned"
    );
}

#[test]
fn every_creature_and_every_temper_survives_the_travel_byte() {
    for id in 0..species::COUNT as u8 {
        for t in 0..TEMPERS {
            let byte = Travel::tempered(SpeciesId(id), t);
            assert_eq!(
                byte.destination(),
                Some(Destination::Hunt(SpeciesId(id), t)),
                "{id} at {t}"
            );
        }
    }
    // The bytes F2 sent still mean what they meant.
    assert_eq!(
        Travel::hunt(SpeciesId::RIDGEBACK).destination(),
        Some(Destination::Hunt(SpeciesId::RIDGEBACK, 0))
    );
    assert_eq!(Travel::VERSUS.destination(), Some(Destination::Versus));
    assert_eq!(Travel::NONE.destination(), None);
    // A temper past the top is the top.
    assert_eq!(
        Travel::tempered(SpeciesId::RIDGEBACK, 200).destination(),
        Some(Destination::Hunt(SpeciesId::RIDGEBACK, HIGHEST))
    );
}

#[test]
fn a_tempered_trip_starts_the_hunt_at_that_temper_and_a_restart_keeps_it() {
    let mut w = World::with_classes(classes());
    let mut ask = [Input::default(); MAX_PLAYERS];
    ask[1] = ask[1].travelling(Travel::tempered(SpeciesId::RIDGEBACK, 2));
    w.advance(ask);
    assert!(w.hunting());
    assert_eq!(w.temper(), 2);
    assert_eq!(w.restarted(classes()).temper(), 2);
    // Stepping to another creature by the plain byte is as tuned.
    let mut next = [Input::default(); MAX_PLAYERS];
    next[0] = next[0].travelling(Travel::hunt(SpeciesId::GNATS));
    w.advance(next);
    assert_eq!(w.temper(), 0);
}

#[test]
fn tempers_never_reach_versus() {
    // Nothing to temper: a versus match comes back from `tempered` unchanged,
    // and a trip to versus out of a tempered hunt is the versus there always
    // was.
    let versus = World::with_classes(classes());
    assert_eq!(versus.clone().tempered(HIGHEST), versus);
    assert_eq!(versus.temper(), 0);

    let mut w = World::hunt(classes()).tempered(HIGHEST);
    let mut back = [Input::default(); MAX_PLAYERS];
    back[0] = back[0].travelling(Travel::VERSUS);
    w.advance(back);
    assert!(!w.hunting());
    assert_eq!(w.temper(), 0);
    let mut fresh = World::with_classes(classes());
    fresh.frame = w.frame;
    assert_eq!(w, fresh);

    // And a versus match plays the same at every setting of the temper knobs:
    // nothing in it reads them.
    let a = run(World::with_classes(classes()), 600);
    let b = run(World::with_classes(classes()).tempered(2), 600);
    assert_eq!(a.checksum(), b.checksum());
    assert_eq!(a.hunt_won(), None);
}

#[test]
fn a_won_hunt_says_what_was_beaten_and_at_which_temper() {
    // The question a trophy is written from (world W1): every creature down.
    let mut w = World::hunt(classes()).tempered(1);
    assert_eq!(w.hunt_won(), None);
    w.monster_mut().unwrap().health = 0;
    w.advance([Input::default(); MAX_PLAYERS]);
    assert!(matches!(w.phase, Phase::RoundOver { winner: 0, .. }));
    let (beaten, at) = w.hunt_won().expect("a dead creature is a won hunt");
    assert_eq!(beaten[0], Some(SpeciesId::RIDGEBACK));
    assert_eq!(at, 1);

    // A lost one is not.
    let mut lost = World::hunt(classes());
    for p in lost.players.iter_mut() {
        p.health = 0;
    }
    lost.advance([Input::default(); MAX_PLAYERS]);
    assert!(matches!(lost.phase, Phase::RoundOver { winner, .. } if winner == QUARRY));
    assert_eq!(lost.hunt_won(), None);
}
