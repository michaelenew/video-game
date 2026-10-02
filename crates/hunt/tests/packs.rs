//! A pack never bites from somewhere it could not be read (bestiary §1.4,
//! `critters.md` §2): the hunts that used to count one, played again.
//!
//! Each seed below dealt an unanswerable bite before a pack's windup begun off
//! a fighter's screen had to land through a marker under them for a reaction
//! (`sim::pack::watch`): the Broodmother's hamstring from behind a pillar, a
//! broodling's crouch below the screen of a hunter looking up at a sac.

use sim::species::SpeciesId;
use sim::state::MAX_PLAYERS;

/// The seed of a `fight --repeats` run, as the binary derives it.
fn run_seed(run: u32) -> u32 {
    0x2545_F491u32.wrapping_add(run.wrapping_mul(0x9E37_79B9))
}

#[test]
fn the_brood_bites_nobody_from_off_the_screen() {
    let card = hunt::plans::card(SpeciesId::BROODMOTHER).expect("its card");
    for (class, run) in [(sim::Class::Champion, 2), (sim::Class::Bulwark, 6)] {
        let report = hunt::play_card_in(
            card,
            None,
            0,
            [class; MAX_PLAYERS],
            1,
            12_000,
            run_seed(run),
            |_| {},
        );
        assert_eq!(
            report.pack.hidden,
            0,
            "{} run {run}: a bite from off the screen landed with no marker under them",
            class.name()
        );
        assert_eq!(report.unanswerable, 0, "{} run {run}", class.name());
    }
}
