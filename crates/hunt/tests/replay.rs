//! A fight recorded as a tape comes back out of the harness as the same fight.
//!
//! The scripted hunter plays a hunt while its inputs are taped; the tape's
//! text is parsed back and judged; the judgement reaches the frame and state
//! the hunt ended on, and its report counts what the hunt's own did.

use hunt::plans;
use sim::replay::Tape;
use sim::species::SpeciesId;
use sim::state::{MAX_PLAYERS, Phase};
use sim::{Class, Input, World};

/// Play a scripted hunt, taping it, and return the tape and the report the
/// live run produced.
fn taped(species: SpeciesId, class: Class, frames: u32, seed: u32) -> (Tape, hunt::Report) {
    let card = plans::card(species).expect("a plan");
    // Not reseeded, unlike `fight`: the game never reseeds a creature, so a
    // game's tape starts from the species' own generator, and the tape's
    // start is checked by hash. The seed goes to the hunter's timing only.
    let mut w = World::hunt_of([class; MAX_PLAYERS], species).seated(1);
    let mut tape = Tape::begin(&w, "test");
    let mut bot = hunt::Hunter::of(card, 0, seed, hunt::jump_apex(class));
    let mut report = hunt::Report::new(card);
    while w.frame < frames && matches!(w.phase, Phase::Fighting) {
        bot.watch(&w);
        let inputs = [bot.act(&w), Input::default()];
        let before = w.clone();
        tape.record(w.frame, inputs);
        w.advance(inputs);
        report.observe(&before, &w, std::slice::from_ref(&bot));
    }
    report.finish(&w);
    tape.finish(&w);
    (tape, report)
}

#[test]
fn a_taped_hunt_replays_to_the_same_end() {
    let (tape, live) = taped(SpeciesId::RIDGEBACK, Class::Champion, 3_000, 7);
    let text = tape.to_text();
    let back = Tape::from_text(&text).expect("the tape parses");
    let judged = hunt::replay::judge(&back).expect("the start rebuilds");
    assert_eq!(
        judged.matched,
        Some(true),
        "{}",
        judged.render(&back, false)
    );
    // The creature's side of the report is the same fight's.
    let report = judged.report.expect("the Ridgeback has a card");
    assert_eq!(report.starts, live.starts);
    assert_eq!(report.landed, live.landed);
    assert_eq!(report.frames, live.frames);
}

#[test]
fn the_hands_count_what_the_hunter_threw() {
    let (tape, _) = taped(SpeciesId::RIDGEBACK, Class::Champion, 3_000, 7);
    let judged = hunt::replay::judge(&tape).expect("the start rebuilds");
    let hands = &judged.hands[0];
    assert_eq!(hands.class, Some(Class::Champion));
    let thrown: u32 = hands.thrown.iter().map(|(_, n)| *n).sum();
    assert!(thrown > 0, "the hunter threw nothing in fifty seconds");
    assert!(hands.frames > 0);
    let text = judged.render(&tape, true);
    assert!(text.contains("WHAT THEY THREW"), "{text}");
    assert!(text.contains("reproduced bit for bit"), "{text}");
}

#[test]
fn a_pack_hunt_replays_too() {
    let (tape, _) = taped(SpeciesId::GNAWERS, Class::Bulwark, 2_000, 3);
    let judged = hunt::replay::judge(&tape).expect("the start rebuilds");
    assert_eq!(judged.matched, Some(true));
}

#[test]
fn a_versus_tape_is_judged_without_a_creature() {
    let w = World::with_classes([Class::Champion, Class::Bulwark]);
    let mut tape = Tape::begin(&w, "test");
    let mut w = w;
    let mut bots = [
        hunt::Duelist::new(0, hunt::Level::Normal, 11),
        hunt::Duelist::new(1, hunt::Level::Normal, 12),
    ];
    for _ in 0..1_800 {
        for b in bots.iter_mut() {
            b.watch(&w);
        }
        let inputs = [bots[0].act(&w), bots[1].act(&w)];
        tape.record(w.frame, inputs);
        w.advance(inputs);
    }
    tape.finish(&w);
    let back = Tape::from_text(&tape.to_text()).expect("parses");
    let judged = hunt::replay::judge(&back).expect("the start rebuilds");
    assert_eq!(judged.matched, Some(true));
    assert!(judged.report.is_none());
    assert_eq!(judged.rounds, w.players.map(|p| p.rounds_won));
    assert!(judged.hands.iter().all(|h| h.frames > 0));
}
