//! The gate on a plain hunt, and across a class change: it has to stay open.
//! It did not (2026-10-15): a live region followed mid-epoch had no aggregate
//! until the epoch's frame, the gate held for it, and the world never reached
//! that frame -- every fight froze four seconds in.

mod common;

use common::Run;
use sim::Class;
use sim::input::{Input, Travel};
use sim::state::MAX_PLAYERS;

fn held(run: &mut Run, ticks: usize) -> usize {
    (0..ticks).filter(|_| !run.tick()).count()
}

#[test]
fn the_gate_stays_open_on_a_hunt_and_across_a_class_change() {
    let mut run = Run::hunt([Class::Champion, Class::Bulwark], 7);
    let before = held(&mut run, 1200);
    assert_eq!(before, 0, "held {before} ticks in the first 1200 frames");
    // The restart, as the local driver does it: the travel rides the input.
    let mut inputs = [Input::default(); MAX_PLAYERS];
    inputs[0] = Input::default().travelling(Travel::class(0, Class::Elementalist));
    run.ledger.tick();
    assert!(run.ledger.may_advance(run.world.frame + 1));
    run.ledger.record(&run.world, inputs);
    let (fed, n) = run.ledger.due(run.world.frame + 1);
    run.world.advance_with(inputs, &fed[..n]);
    run.ledger.confirm_through(run.world.frame, &run.world);
    assert_eq!(run.world.players[0].class, Class::Elementalist);
    let after = held(&mut run, 1200);
    assert_eq!(
        after, 0,
        "held {after} ticks in the 1200 frames after a class change"
    );
}
