//! The four windows are each body's own, and asked of the frames they are
//! about (`hunt::report`; `the-pair.md` §9, `galewing.md` §9).

use hunt::report::{Tally, Threat};
use sim::monster::Doing;
use sim::species::SpeciesId;
use sim::species::galewing as gw;
use sim::state::MAX_PLAYERS;

#[test]
fn a_pairs_windows_are_each_cats_own_and_together_is_kept_beside_them() {
    // Two cats, each frame counted once for each that is alive: more counts
    // than frames fought. Together -- free when either is -- is at least as
    // threatening as either cat's own, which is what it used to report.
    let card = hunt::plans::card(SpeciesId::PAIR).expect("its card");
    let r = hunt::play_card_in(
        card,
        None,
        0,
        [sim::Class::Champion; MAX_PLAYERS],
        1,
        6_000,
        7,
        |_| {},
    );
    let each: u32 = r.threat.iter().sum();
    let together: u32 = r.together.iter().sum();
    assert!(
        each > together,
        "two cats should count twice: {each} against {together}"
    );
    let share =
        |c: &[u32; 5]| c[Threat::Threatening as usize] as f32 / c.iter().sum::<u32>() as f32;
    assert!(
        share(&r.together) >= share(&r.threat),
        "either cat free is at least as threatening as one: {} against {}",
        share(&r.together),
        share(&r.threat)
    );
}

#[test]
fn a_bird_gathering_itself_off_the_floor_is_not_a_threat() {
    // The gather and the lift do no damage and move nobody: until it can
    // next hit is the rest of the move, its recovery and its pause.
    let mut w = sim::World::hunt_of([sim::Class::Champion; MAX_PLAYERS], SpeciesId::GALEWING);
    let slot = gw::fight::slot_of(&w).expect("the bird");
    let a = gw::SPECIES.attack(gw::LIFT);
    let m = w.monsters[slot].as_mut().unwrap();
    m.doing = Doing::Startup {
        kind: gw::LIFT,
        left: a.startup,
    };
    let tally = hunt::plans::galewing::GaleTally::default();
    let free = tally.until_free(&w, slot, 0);
    assert!(
        free >= a.startup + a.active + a.recovery,
        "a lift counted as a threat: {free} frames until free"
    );
    assert_ne!(hunt::report::Threat::of(free as i32), Threat::Threatening);
}

#[test]
fn the_galewings_windows_leave_out_the_frames_it_is_out_of_reach() {
    let card = hunt::plans::card(SpeciesId::GALEWING).expect("its card");
    let r = hunt::play_card_in(
        card,
        None,
        0,
        [sim::Class::Champion; MAX_PLAYERS],
        1,
        6_000,
        7,
        |_| {},
    );
    let counted: u32 = r.threat.iter().sum();
    assert!(
        counted < r.fought,
        "every fought frame was windowed ({counted} of {}): a circling bird out of reach is in none",
        r.fought
    );
    assert!(counted > 0, "no frame was in reach in a hundred seconds");
}
