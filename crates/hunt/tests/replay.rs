//! A fight recorded as a tape comes back out of the harness as the same fight.
//!
//! The scripted hunter plays a hunt while its inputs are taped; the tape's
//! text is parsed back and judged; the judgement reaches the frame and state
//! the hunt ended on, and its report counts what the hunt's own did.

use hunt::plans;
use sim::replay::Tape;
use sim::species::SpeciesId;
use sim::state::Phase;
use sim::{Class, Input, World};

/// One frame as the game's local driver makes it: the books kept beside
/// the world, and the crowd's aggregates due fed to it. A tape is judged fed
/// the same way (`regions::Ledger::for_tape`), so a fight made any other way
/// is not the fight its tape describes.
fn kept(w: &mut World, books: &mut regions::Ledger, inputs: [Input; 2]) {
    let mut all = [Input::default(); sim::state::MAX_PLAYERS];
    all[..2].copy_from_slice(&inputs);
    books.record(w, all);
    let (due, n) = books.due(w.frame + 1);
    w.advance_with(inputs, &due[..n]);
    books.confirm_through(w.frame, w);
}

/// Play a scripted hunt, taping it, and return the tape and the report the
/// live run produced.
fn taped(species: SpeciesId, class: Class, frames: u32, seed: u32) -> (Tape, hunt::Report) {
    let card = plans::card(species).expect("a plan");
    // Not reseeded, unlike `fight`: the game never reseeds a creature, so a
    // game's tape starts from the species' own generator, and the tape's
    // start is checked by hash. The seed goes to the hunter's timing only.
    let mut w = World::hunt_of([class; 2], species).seated(1);
    let mut tape = Tape::begin(&w, "test");
    let mut books = regions::Ledger::for_tape(&tape, &w, false);
    let mut bot = hunt::Hunter::of(card, 0, seed, hunt::jump_apex(class));
    let mut report = hunt::Report::new(card);
    while w.frame < frames && matches!(w.phase, Phase::Fighting) {
        bot.watch(&w);
        let inputs = [bot.act(&w), Input::default()];
        let before = w.clone();
        tape.record(w.frame, inputs);
        kept(&mut w, &mut books, inputs);
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
    assert_eq!(hands.stints.len(), 1, "one class, one table");
    assert_eq!(hands.stints[0].class, Class::Champion);
    let thrown: u32 = hands.stints[0].thrown.iter().map(|(_, n)| *n).sum();
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
    let mut books = regions::Ledger::for_tape(&tape, &w, false);
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
        kept(&mut w, &mut books, inputs);
    }
    tape.finish(&w);
    let back = Tape::from_text(&tape.to_text()).expect("parses");
    let judged = hunt::replay::judge(&back).expect("the start rebuilds");
    assert_eq!(judged.matched, Some(true));
    assert!(judged.report.is_none());
    assert_eq!(judged.rounds, w.players.map(|p| p.rounds_won));
    assert!(judged.hands[..w.present()].iter().all(|h| h.frames > 0));
}

#[test]
fn a_taped_walk_through_the_valley_reports_its_places() {
    // Two people walk out of Hearth's square and through the valley gate
    // together, and on up the Mouth. The judgement names both places and
    // says they went on together.
    let mut w = World::versus_in([Class::Champion, Class::Elementalist], sim::valley::START);
    assert!(w.valley.on);
    let mut tape = Tape::begin(&w, "test");
    let mut books = regions::Ledger::for_tape(&tape, &w, false);
    let walk = Input::aimed(Input::W, 0);
    for _ in 0..600 {
        let inputs = [walk, walk];
        tape.record(w.frame, inputs);
        kept(&mut w, &mut books, inputs);
    }
    tape.finish(&w);
    let back = Tape::from_text(&tape.to_text()).expect("the tape parses");
    let judged = hunt::replay::judge(&back).expect("the start rebuilds");
    assert_eq!(
        judged.matched,
        Some(true),
        "{}",
        judged.render(&back, false)
    );
    let legs = &judged.trek.legs;
    assert!(legs.len() >= 2, "{}", judged.render(&back, false));
    assert_eq!(legs[0].arena, sim::arena::ArenaId::HEARTH);
    assert_eq!(
        legs[0].out,
        Some(("the valley", true)),
        "{}",
        judged.render(&back, false)
    );
    assert_eq!(legs[1].arena, sim::arena::ArenaId::MOUTH);
    assert!(judged.render(&back, false).contains("THE VALLEY"));
}

/// Two fighters on the floor, metres apart.
fn apart(w: &World) -> f32 {
    let d = w.players[0].pos.sub(w.players[1].pos);
    sim::math::wide_len(sim::V3::new(d.x, sim::Fx::ZERO, d.z)).to_f32_for_render()
}

#[test]
fn the_link_is_laid_beside_how_far_apart_the_fighters_were() {
    // Two people online in the valley walk away from each other. The line
    // stays quick, but once they are past forty metres this machine waits
    // on the other one: the friend's machine falling behind, out there.
    let mut w = World::versus_in([Class::Champion, Class::Elementalist], sim::valley::START);
    let mut tape = Tape::begin(&w, "test");
    tape.online = Some((0, 0));
    let mut books = regions::Ledger::for_tape(&tape, &w, false);
    let (out, back) = (
        Input::aimed(Input::W, 0),
        Input::aimed(Input::W, 2 * Input::QUARTER_TURN),
    );
    let mut far = 0;
    for f in 0..2_400 {
        // Twenty seconds side by side, then away from each other.
        let inputs = if f < 1_200 {
            [Input::default(); 2]
        } else {
            [out, back]
        };
        tape.record(w.frame, inputs);
        kept(&mut w, &mut books, inputs);
        if f % 60 == 59 {
            let distant = apart(&w) > 40.0;
            far += u32::from(distant);
            tape.measure(sim::replay::Link {
                frame: w.frame,
                ping: 40,
                rollbacks: 3,
                resimulated: 9,
                deepest: 4,
                stalls: if distant { 25 } else { 0 },
                slowest: 17,
                ..Default::default()
            });
        }
    }
    tape.finish(&w);
    assert!(far >= 10, "they never got far apart: {} m", apart(&w));
    assert!(far <= 30, "they were never close: {far} s apart");
    let back = Tape::from_text(&tape.to_text()).expect("the tape parses");
    let judged = hunt::replay::judge(&back).expect("the start rebuilds");
    assert_eq!(judged.matched, Some(true));
    assert_eq!(judged.lag.seconds.len(), 40);
    let text = judged.render(&back, false);
    assert!(text.contains("THE LINK  as player 1's machine"), "{text}");
    assert!(text.contains("ROUGH STRETCHES"), "{text}");
    assert!(text.contains("waiting on the friend's machine"), "{text}");
    assert!(text.contains("falling behind"), "{text}");
}

#[test]
fn a_tape_with_no_link_still_says_what_the_simulation_costs_apart() {
    let w = World::versus_in([Class::Champion, Class::Elementalist], sim::valley::START);
    let mut tape = Tape::begin(&w, "test");
    let mut books = regions::Ledger::for_tape(&tape, &w, false);
    let mut w = w;
    for _ in 0..300 {
        let inputs = [Input::aimed(Input::W, 0), Input::default()];
        tape.record(w.frame, inputs);
        kept(&mut w, &mut books, inputs);
    }
    let judged = hunt::replay::judge(&tape).expect("the start rebuilds");
    let text = judged.render(&tape, false);
    assert!(text.contains("THE LINK  not on this tape"), "{text}");
    assert!(text.contains("BY DISTANCE APART"), "{text}");
}

/// A class changed in the middle of a tape gets its own table, and the moves
/// thrown before the change are named by the class that threw them -- a move
/// is a slot number, and the same slot is a different move on every class.
#[test]
fn a_change_of_class_gets_its_own_table() {
    let mut w = World::hunt_of([Class::Bulwark; 2], SpeciesId::RIDGEBACK).seated(1);
    let mut tape = Tape::begin(&w, "test");
    let mut books = regions::Ledger::for_tape(&tape, &w, false);
    let swing = Input {
        bits: Input::LEFT,
        ..Input::default()
    };
    for f in 0..240u32 {
        let mut mine = if f % 40 == 0 || f % 40 == 1 {
            swing
        } else {
            Input::default()
        };
        if f == 120 {
            mine.travel = sim::input::Travel::class(0, Class::Champion);
        }
        let inputs = [mine, Input::default()];
        tape.record(w.frame, inputs);
        kept(&mut w, &mut books, inputs);
    }
    tape.finish(&w);
    let judged = hunt::replay::judge(&tape).expect("the start rebuilds");
    let hands = &judged.hands[0];
    let classes: Vec<Class> = hands.stints.iter().map(|s| s.class).collect();
    assert_eq!(classes, [Class::Bulwark, Class::Champion]);
    assert!(hands.stints.iter().all(|s| s.frames > 0));
    assert_eq!(
        hands.stints.iter().map(|s| s.frames).sum::<u32>(),
        hands.frames
    );
    let text = judged.render(&tape, false);
    assert!(
        text.contains("Bulwark") && text.contains("Champion"),
        "{text}"
    );
}

/// **A match's tape reproduces only fed as the match was fed.** The crowd
/// field's aggregates are fed to the world beside the inputs, off books
/// begun at the agreed start, and they are not on the tape: the judge keeps
/// the same books. Played without them -- as it was until 2026-10-24 --
/// the owner's tape of a versus match in the Needle came out a different
/// fight, and so did every tape the game had saved since the crowd field.
#[test]
fn a_match_tape_is_played_fed_as_the_match_was() {
    let needle = sim::arena::named("needle").expect("the Needle").id;
    let w = World::versus_in([Class::ShadowReaver, Class::Champion], needle).drafting(true);
    let mut tape = Tape::begin(&w, "test");
    tape.online = Some((1, 2));
    let mut books = regions::Ledger::for_tape(&tape, &w, false);
    let mut w = w;
    let mut bots = [
        hunt::Duelist::new(0, hunt::Level::Normal, 21),
        hunt::Duelist::new(1, hunt::Level::Normal, 22),
    ];
    for _ in 0..1_200 {
        for b in bots.iter_mut() {
            b.watch(&w);
        }
        let inputs = [bots[0].act(&w), bots[1].act(&w)];
        tape.record(w.frame, inputs);
        kept(&mut w, &mut books, inputs);
    }
    tape.finish(&w);
    let back = Tape::from_text(&tape.to_text()).expect("parses");
    let judged = hunt::replay::judge(&back).expect("the start rebuilds");
    assert_eq!(
        judged.matched,
        Some(true),
        "{}",
        judged.render(&back, false)
    );
    let unfed = back
        .play(&mut sim::replay::Unfed, |_, _, _| {})
        .expect("the start rebuilds");
    assert_ne!(
        back.matched(&unfed),
        Some(true),
        "nothing was fed in twenty seconds: the test proves nothing"
    );
}
