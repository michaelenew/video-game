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

#[test]
fn a_taped_walk_through_the_valley_reports_its_places() {
    // Two people walk out of Hearth's square and through the valley gate
    // together, and on up the Mouth. The judgement names both places and
    // says they went on together.
    let mut w = World::versus_in([Class::Champion, Class::Elementalist], sim::valley::START);
    assert!(w.valley.on);
    let mut tape = Tape::begin(&w, "test");
    let walk = Input::aimed(Input::W, 0);
    for _ in 0..600 {
        let inputs = [walk, walk];
        tape.record(w.frame, inputs);
        w.advance(inputs);
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
    let (out, back) = (
        Input::aimed(Input::W, 0),
        Input::aimed(Input::W, 2 * Input::QUARTER_TURN),
    );
    let mut far = 0;
    for f in 0..2_400 {
        // Twenty seconds side by side, then away from each other.
        let inputs = if f < 1_200 {
            [Input::default(); MAX_PLAYERS]
        } else {
            [out, back]
        };
        tape.record(w.frame, inputs);
        w.advance(inputs);
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
    let mut w = w;
    for _ in 0..300 {
        let inputs = [Input::aimed(Input::W, 0), Input::default()];
        tape.record(w.frame, inputs);
        w.advance(inputs);
    }
    let judged = hunt::replay::judge(&tape).expect("the start rebuilds");
    let text = judged.render(&tape, false);
    assert!(text.contains("THE LINK  not on this tape"), "{text}");
    assert!(text.contains("BY DISTANCE APART"), "{text}");
}
