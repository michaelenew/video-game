//! The gate and the simulated neighbours' heartbeats: `docs/design/regions.md`
//! §"The ledger". One test, because it moves Oven knobs and the Oven is shared
//! by everything in the process.

mod common;

use common::Run;
use regions::Finding;
use sim::Class;
use sim::oven::{self, Scalar};

#[test]
fn a_neighbour_slower_than_the_delay_holds_the_game_then_paces_it() {
    // As tuned the lag is under the delay and nothing is ever held.
    let mut run = Run::hunt([Class::Champion, Class::Bulwark], 7);
    for _ in 0..120 {
        assert!(run.tick(), "held at frame {}", run.world.frame);
    }
    assert!(run.ledger.live().len() == 3, "{:?}", run.ledger.live());
    assert!(run.ledger.heard().iter().any(|h| h.simulated));
    assert!(run.ledger.heard().iter().any(|h| !h.simulated));

    // Five frames past the delay: held for five ticks, then one frame a tick,
    // paced by the neighbour's heartbeat -- what a slow peer does to you.
    let delay = sim::tuning::region_delay();
    oven::set_scalar(Scalar::RegionNeighbourLag, delay as i32 + 5);
    let mut run = Run::hunt([Class::Champion, Class::Bulwark], 7);
    let advanced = (0..200).filter(|_| run.tick()).count();
    oven::reset_to_baked();
    assert_eq!(advanced, 195, "five ticks held, every other tick a frame");
    let held: Vec<_> = run
        .ledger
        .watch()
        .unwrap()
        .issues()
        .filter_map(|i| match i.finding {
            Finding::Held { ticks, .. } => Some(ticks),
            _ => None,
        })
        .collect();
    assert_eq!(held, [5], "one hold, five ticks long");
}
